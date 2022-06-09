//! Field arithmetic macros

// TODO(tarcieri): extract this into the `elliptic-curve` crate when stable

/// Provides both inherent and trait impls for a field element type which are
/// backed by a core set of arithmetic functions specified as macro arguments.
///
/// # Inherent impls
/// - `const ZERO: Self`
/// - `const ONE: Self` (multiplicative identity)
/// - `pub fn from_be_bytes`
/// - `pub fn from_be_slice`
/// - `pub fn from_le_bytes`
/// - `pub fn from_le_slice`
/// - `pub fn from_uint`
/// - `fn from_uint_unchecked`
/// - `pub fn to_be_bytes`
/// - `pub fn to_le_bytes`
/// - `pub fn to_canonical`
/// - `pub fn is_odd`
/// - `pub fn is_zero`
/// - `pub fn double`
///
/// NOTE: field implementations must provide their own inherent impls of
/// the following methods in order for the code generated by this macro to
/// compile:
///
/// - `pub fn invert`
/// - `pub fn sqrt`
///
/// # Trait impls
/// - `AsRef<$arr>`
/// - `ConditionallySelectable`
/// - `ConstantTimeEq`
/// - `ConstantTimeGreater`
/// - `ConstantTimeLess`
/// - `Default`
/// - `DefaultIsZeroes`
/// - `Eq`
/// - `Field`
/// - `PartialEq`
///
/// ## Ops
/// - `Add`
/// - `AddAssign`
/// - `Sub`
/// - `SubAssign`
/// - `Mul`
/// - `MulAssign`
/// - `Neg`
macro_rules! impl_field_element {
    (
        $fe:tt,
        $bytes:ty,
        $uint:ty,
        $modulus:expr,
        $arr:ty,
        $from_mont:ident,
        $to_mont:ident,
        $add:ident,
        $sub:ident,
        $mul:ident,
        $neg:ident,
        $square:ident
    ) => {
        impl $fe {
            /// Zero element.
            pub const ZERO: Self = Self(<$uint>::ZERO);

            /// Multiplicative identity.
            pub const ONE: Self = Self::from_uint_unchecked(<$uint>::ONE);

            /// Create a [`
            #[doc = stringify!($fe)]
            /// `] from a canonical big-endian representation.
            pub fn from_be_bytes(repr: $bytes) -> ::elliptic_curve::subtle::CtOption<Self> {
                Self::from_uint(<$uint>::from_be_byte_array(repr))
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `] from a big endian byte slice.
            pub fn from_be_slice(slice: &[u8]) -> ::elliptic_curve::Result<Self> {
                <$uint as ::elliptic_curve::bigint::Encoding>::Repr::try_from(slice)
                    .ok()
                    .and_then(|array| Self::from_be_bytes(array.into()).into())
                    .ok_or(::elliptic_curve::Error)
            }

            /// Create a [`
            #[doc = stringify!($fe)]
            /// `] from a canonical little-endian representation.
            pub fn from_le_bytes(repr: $bytes) -> ::elliptic_curve::subtle::CtOption<Self> {
                Self::from_uint(<$uint>::from_le_byte_array(repr))
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `] from a little endian byte slice.
            pub fn from_le_slice(slice: &[u8]) -> ::elliptic_curve::Result<Self> {
                <$uint as Encoding>::Repr::try_from(slice)
                    .ok()
                    .and_then(|array| Self::from_le_bytes(array.into()).into())
                    .ok_or(::elliptic_curve::Error)
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `]
            /// from [`
            #[doc = stringify!($uint)]
            /// `] converting it into Montgomery form:
            ///
            /// ```text
            /// w * R^2 * R^-1 mod p = wR mod p
            /// ```
            pub fn from_uint(uint: $uint) -> ::elliptic_curve::subtle::CtOption<Self> {
                let is_some = uint.ct_lt(&$modulus);
                ::elliptic_curve::subtle::CtOption::new(Self::from_uint_unchecked(uint), is_some)
            }

            /// Parse a [`
            #[doc = stringify!($fe)]
            /// `] from big endian hex-encoded bytes.
            ///
            /// Does *not* perform a check that the field element does not overflow the order.
            ///
            /// This method is primarily intended for defining internal constants.
            #[allow(dead_code)]
            pub(crate) const fn from_be_hex(hex: &str) -> Self {
                Self::from_uint_unchecked(<$uint>::from_be_hex(hex))
            }

            /// Parse a [`
            #[doc = stringify!($fe)]
            /// `] from little endian hex-encoded bytes.
            ///
            /// Does *not* perform a check that the field element does not overflow the order.
            ///
            /// This method is primarily intended for defining internal constants.
            #[allow(dead_code)]
            pub(crate) const fn from_le_hex(hex: &str) -> Self {
                Self::from_uint_unchecked(<$uint>::from_le_hex(hex))
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `] from [`
            #[doc = stringify!($uint)]
            /// `] converting it into Montgomery form.
            ///
            /// Does *not* perform a check that the field element does not overflow the order.
            ///
            /// Used incorrectly this can lead to invalid results!
            const fn from_uint_unchecked(w: $uint) -> Self {
                Self(<$uint>::from_uint_array($to_mont(w.as_uint_array())))
            }

            /// Returns the big-endian encoding of this [`
            #[doc = stringify!($fe)]
            /// `].
            pub fn to_be_bytes(self) -> $bytes {
                self.to_canonical().to_be_byte_array()
            }

            /// Returns the little-endian encoding of this [`
            #[doc = stringify!($fe)]
            /// `].
            pub fn to_le_bytes(self) -> $bytes {
                self.to_canonical().to_le_byte_array()
            }

            /// Translate [`
            #[doc = stringify!($fe)]
            /// `] out of the Montgomery domain, returning a [`
            #[doc = stringify!($uint)]
            /// `] in canonical form.
            #[inline]
            pub const fn to_canonical(self) -> $uint {
                <$uint>::from_uint_array($from_mont(self.0.as_uint_array()))
            }

            /// Determine if this [`
            #[doc = stringify!($fe)]
            /// `] is odd in the SEC1 sense: `self mod 2 == 1`.
            ///
            /// # Returns
            ///
            /// If odd, return `Choice(1)`.  Otherwise, return `Choice(0)`.
            pub fn is_odd(&self) -> Choice {
                self.to_canonical().is_odd()
            }

            /// Determine if this [`
            #[doc = stringify!($fe)]
            /// `] is zero.
            ///
            /// # Returns
            ///
            /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
            pub fn is_zero(&self) -> Choice {
                self.ct_eq(&Self::ZERO)
            }

            /// Add elements.
            pub const fn add(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_uint_array($add(
                    self.0.as_uint_array(),
                    rhs.0.as_uint_array(),
                )))
            }

            /// Double element (add it to itself).
            #[must_use]
            pub const fn double(&self) -> Self {
                self.add(self)
            }

            /// Subtract elements.
            pub const fn sub(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_uint_array($sub(
                    self.0.as_uint_array(),
                    rhs.0.as_uint_array(),
                )))
            }

            /// Multiply elements.
            pub const fn mul(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_uint_array($mul(
                    self.0.as_uint_array(),
                    rhs.0.as_uint_array(),
                )))
            }

            /// Negate element.
            pub const fn neg(&self) -> Self {
                Self(<$uint>::from_uint_array($neg(self.0.as_uint_array())))
            }

            /// Compute modular square.
            #[must_use]
            pub const fn square(&self) -> Self {
                Self(<$uint>::from_uint_array($square(self.0.as_uint_array())))
            }
        }

        impl AsRef<$arr> for $fe {
            fn as_ref(&self) -> &$arr {
                self.0.as_ref()
            }
        }

        impl Default for $fe {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl Eq for $fe {}

        impl PartialEq for $fe {
            fn eq(&self, rhs: &Self) -> bool {
                self.0.ct_eq(&(rhs.0)).into()
            }
        }

        impl ::elliptic_curve::subtle::ConditionallySelectable for $fe {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                Self(<$uint>::conditional_select(&a.0, &b.0, choice))
            }
        }

        impl ::elliptic_curve::subtle::ConstantTimeEq for $fe {
            fn ct_eq(&self, other: &Self) -> ::elliptic_curve::subtle::Choice {
                self.0.ct_eq(&other.0)
            }
        }

        impl ::elliptic_curve::subtle::ConstantTimeGreater for $fe {
            fn ct_gt(&self, other: &Self) -> ::elliptic_curve::subtle::Choice {
                self.0.ct_gt(&other.0)
            }
        }

        impl ::elliptic_curve::subtle::ConstantTimeLess for $fe {
            fn ct_lt(&self, other: &Self) -> ::elliptic_curve::subtle::Choice {
                self.0.ct_lt(&other.0)
            }
        }

        impl ::elliptic_curve::zeroize::DefaultIsZeroes for $fe {}

        impl ::elliptic_curve::ff::Field for $fe {
            fn random(mut rng: impl ::elliptic_curve::rand_core::RngCore) -> Self {
                // NOTE: can't use ScalarCore::random due to CryptoRng bound
                let mut bytes = <$bytes>::default();

                loop {
                    rng.fill_bytes(&mut bytes);
                    if let Some(fe) = Self::from_be_bytes(bytes).into() {
                        return fe;
                    }
                }
            }

            fn zero() -> Self {
                Self::ZERO
            }

            fn one() -> Self {
                Self::ONE
            }

            fn is_zero(&self) -> Choice {
                Self::ZERO.ct_eq(self)
            }

            #[must_use]
            fn square(&self) -> Self {
                self.square()
            }

            #[must_use]
            fn double(&self) -> Self {
                self.double()
            }

            fn invert(&self) -> CtOption<Self> {
                self.invert()
            }

            fn sqrt(&self) -> CtOption<Self> {
                self.sqrt()
            }
        }

        impl_field_op!($fe, $uint, Add, add, $add);
        impl_field_op!($fe, $uint, Sub, sub, $sub);
        impl_field_op!($fe, $uint, Mul, mul, $mul);

        impl AddAssign<$fe> for $fe {
            #[inline]
            fn add_assign(&mut self, other: $fe) {
                *self = *self + other;
            }
        }

        impl AddAssign<&$fe> for $fe {
            #[inline]
            fn add_assign(&mut self, other: &$fe) {
                *self = *self + other;
            }
        }

        impl SubAssign<$fe> for $fe {
            #[inline]
            fn sub_assign(&mut self, other: $fe) {
                *self = *self - other;
            }
        }

        impl SubAssign<&$fe> for $fe {
            #[inline]
            fn sub_assign(&mut self, other: &$fe) {
                *self = *self - other;
            }
        }

        impl MulAssign<&$fe> for $fe {
            #[inline]
            fn mul_assign(&mut self, other: &$fe) {
                *self = *self * other;
            }
        }

        impl MulAssign for $fe {
            #[inline]
            fn mul_assign(&mut self, other: $fe) {
                *self = *self * other;
            }
        }

        impl Neg for $fe {
            type Output = $fe;

            #[inline]
            fn neg(self) -> $fe {
                Self($neg(self.as_ref()).into())
            }
        }
    };
}

/// Emit impls for a `core::ops` trait for all combinations of reference types,
/// which thunk to the given function.
macro_rules! impl_field_op {
    ($fe:tt, $uint:ty, $op:tt, $op_fn:ident, $func:ident) => {
        impl ::core::ops::$op for $fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: $fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }

        impl ::core::ops::$op<&$fe> for $fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: &$fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }

        impl ::core::ops::$op<&$fe> for &$fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: &$fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }
    };
}

/// Implement field element inversion.
macro_rules! impl_field_invert {
    (
        $a:expr,
        $one:expr,
        $word_bits:expr,
        $nlimbs:expr,
        $mul:ident,
        $neg:ident,
        $divstep_precomp:ident,
        $divstep:ident,
        $msat:ident,
        $selectznz:ident,
    ) => {{
        const ITERATIONS: usize = (49 * $nlimbs * $word_bits + 57) / 17;

        let mut d = 1;
        let mut f = $msat();
        let mut g = [0; $nlimbs + 1];
        let mut v = Default::default();
        let mut r = $one;
        let mut i = 0;

        g[..$nlimbs].copy_from_slice($a.as_ref());

        while i < ITERATIONS - ITERATIONS % 2 {
            let (out1, out2, out3, out4, out5) = $divstep(d, &f, &g, &v, &r);
            let (out1, out2, out3, out4, out5) = $divstep(out1, &out2, &out3, &out4, &out5);
            d = out1;
            f = out2;
            g = out3;
            v = out4;
            r = out5;
            i += 2;
        }

        if ITERATIONS % 2 != 0 {
            let (_out1, out2, _out3, out4, _out5) = $divstep(d, &f, &g, &v, &r);
            v = out4;
            f = out2;
        }

        let s = ((f[f.len() - 1] >> $word_bits - 1) & 1) as u8;
        let v = $selectznz(s, &v, &$neg(&v));
        $mul(&v, &$divstep_precomp())
    }};
}
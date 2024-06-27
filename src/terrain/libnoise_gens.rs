use libnoise::*;

/* Trait for Dynamic Dispatch */
pub trait Noise2 {
    fn sample(&self, point: [f64; 2]) -> f64;
}

impl<T> Noise2 for T where
    T : Generator2D + ExtraGenerator<2>
{
    fn sample(&self, point: [f64; 2]) -> f64 {
        self.sample(point)
    }
}

pub trait Noise3 {
    fn sample(&self, point: [f64; 3]) -> f64;
}

impl<T> Noise3 for T where
    T : Generator3D + ExtraGenerator<3>
{
    fn sample(&self, point: [f64; 3]) -> f64 {
        self.sample(point)
    }
}

/* LambdaPoint stuff, perhaps we should put this upstream, it seems useful */
// extension trait for any generator functions i deign to implement myself
pub trait ExtraGenerator<const D: usize> : Sized {
    fn lambda_point<L>(self, lambda: L) -> LambdaPoint<D, Self, L>
    where L: Fn([f64; D], f64) -> f64;
}

impl<const D: usize, T> ExtraGenerator<D> for T where T : Generator<D> {
    #[inline]
    fn lambda_point<L>(self, lambda: L) -> LambdaPoint<D, Self, L>
    where
        L: Fn([f64; D], f64) -> f64
    {
        LambdaPoint::new(self, lambda)
    }
}

/* Lambda that also takes in the point, for use in density scaling and whatnot */
#[derive(Clone, Copy, Debug)]
pub struct LambdaPoint<const D: usize, G, L> {
    generator: G,
    lambda: L,
}

impl<G: Generator<1>, L: Fn([f64; 1], f64) -> f64 + Copy> Generator1D for LambdaPoint<1, G, L> {}
impl<G: Generator<2>, L: Fn([f64; 2], f64) -> f64 + Copy> Generator2D for LambdaPoint<2, G, L> {}
impl<G: Generator<3>, L: Fn([f64; 3], f64) -> f64 + Copy> Generator3D for LambdaPoint<3, G, L> {}
impl<G: Generator<4>, L: Fn([f64; 4], f64) -> f64 + Copy> Generator4D for LambdaPoint<4, G, L> {}

impl<const D: usize, G, L> LambdaPoint<D, G, L>
where
    G: Generator<D>,
    L: Fn([f64; D], f64) -> f64,
{
    #[inline]
    pub fn new(generator: G, lambda: L) -> Self {
        Self { generator, lambda}
    }
}

impl<const D: usize, G, L> Generator<D> for LambdaPoint<D, G, L>
where
    G: Generator<D>,
    L: Copy + Fn([f64; D], f64) -> f64,
{
    #[inline]
    fn sample(&self, point: [f64; D]) -> f64 {
        (self.lambda)(point, self.generator.sample(point))
    }
}
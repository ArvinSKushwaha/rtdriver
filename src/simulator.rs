use crate::{vector::Vector, Float, STENCIL};

#[derive(Debug, Clone, PartialEq)]
pub struct Simulation<T: Float, const SIZE: usize> {
    state: SimulationState<T, SIZE>,
    tmp_acc: Box<[[Vector<T>; SIZE]; SIZE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationState<T: Float, const SIZE: usize> {
    stiffness: T,
    origin_stiffness: T,
    pos: Box<[[Vector<T>; SIZE]; SIZE]>,
    vel: Box<[[Vector<T>; SIZE]; SIZE]>,
    acc: Box<[[Vector<T>; SIZE]; SIZE]>,
}

pub struct SimulationBuilder<T, const SIZE: usize> {
    stiffness: Option<T>,
    origin_stiffness: Option<T>,
}

impl<T: Float, const SIZE: usize> SimulationBuilder<T, SIZE> {
    pub fn stiffness(mut self, stiffness: T) -> Self {
        self.stiffness.replace(stiffness);
        self
    }

    pub fn origin_stiffness(mut self, origin_stiffness: T) -> Self {
        self.origin_stiffness.replace(origin_stiffness);
        self
    }

    pub fn finish(self) -> Simulation<T, SIZE> {
        let Self {
            stiffness,
            origin_stiffness,
        } = self;
        let stiffness = stiffness.unwrap_or(T::one());
        let origin_stiffness = origin_stiffness.unwrap_or(T::one());

        let pos = bytemuck::zeroed_box();
        let vel = bytemuck::zeroed_box();
        let acc = bytemuck::zeroed_box();
        let tmp_acc = bytemuck::zeroed_box();

        Simulation {
            tmp_acc,
            state: SimulationState {
                pos,
                vel,
                acc,
                stiffness,
                origin_stiffness,
            },
        }
    }
}

fn filter_indices<const SIZE: usize, const DIMS: usize>(
    indices: Vector<isize, DIMS>,
) -> Option<Vector<usize, DIMS>> {
    if indices.map(|i| (0..SIZE as isize).contains(&i)).all() {
        Some(indices.map(|i| i as usize))
    } else {
        None
    }
}

fn index<const SIZE: usize, const DIMS: usize>(indices: Vector<isize, DIMS>) -> Option<usize> {
    if let Some(indices) = filter_indices::<SIZE, DIMS>(indices) {
        let data = Vector::from_idx(|i| SIZE.pow((DIMS - 1 - i) as u32));
        Some((indices * data).sum())
    } else {
        None
    }
}

fn deindex<const SIZE: usize, const DIMS: usize>(k: isize) -> Option<Vector<usize, DIMS>> {
    let range = 0..(SIZE as isize).pow(DIMS as u32);
    if range.contains(&k) {
        let k = Vector::broadcast(k as usize);
        let data = Vector::from_idx(|i| SIZE.pow((DIMS - 1 - i) as u32));
        Some((k / data) % Vector::broadcast(SIZE))
    } else {
        None
    }
}

impl<T: Float, const SIZE: usize> Simulation<T, SIZE> {
    pub fn build() -> SimulationBuilder<T, SIZE> {
        SimulationBuilder {
            stiffness: None,
            origin_stiffness: None,
        }
    }

    pub fn update(&mut self, dt: T) {
        self.compute_acc();
        std::mem::swap(&mut self.tmp_acc, &mut self.state.acc);
    }

    fn compute_acc(&mut self) {
        let SimulationState {
            stiffness,
            origin_stiffness,
            pos,
            vel: _,
            acc: _,
        } = &mut self.state;
        let tmp_acc = &mut self.tmp_acc;

        for i in 0..SIZE {
            for j in 0..SIZE {
                let indices = Vector([i as isize, j as isize]);

                let position_here = pos[i][j];
                let origin_acc = -position_here.map(|i| i * *origin_stiffness);
                let mut coupled_acc: Vector<T, { crate::DIMS }> = Vector::zero();

                for indices in STENCIL
                    .map(|[stencil_up, stencil_down]| {
                        let indices_up = indices + stencil_up;
                        let indices_down = indices + stencil_down;

                        [
                            filter_indices::<SIZE, { crate::DIMS }>(indices_up),
                            filter_indices::<SIZE, { crate::DIMS }>(indices_down),
                        ]
                    })
                    .into_iter()
                    .flatten()
                    .flatten()
                {
                    let position_stencil = pos[indices[0]][indices[1]];
                    coupled_acc = coupled_acc - position_stencil.map(|i| i * *stiffness);
                }

                tmp_acc[i][j] = origin_acc + coupled_acc;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        simulator::{deindex, index},
        Vector,
    };

    #[test]
    fn test_index() {
        assert_eq!(index::<100, 2>(Vector([2, 10])), Some(210));
        assert_eq!(index::<10, 2>(Vector([2, 10])), None);
        assert_eq!(index::<10, 2>(Vector([10, 2])), None);
        assert_eq!(index::<10, 2>(Vector([9, 9])), Some(99));
        assert_eq!(index::<10, 2>(Vector([0, 0])), Some(0));
        assert_eq!(index::<10, 2>(Vector([-1, 2])), None);
        assert_eq!(index::<10, 2>(Vector([2, -1])), None);
        assert_eq!(index::<10, 2>(Vector([-1, -1])), None);

        assert_eq!(index::<100, 3>(Vector([2, 10, 5])), Some(21_005));
        assert_eq!(index::<10, 3>(Vector([2, 10, 2])), None);
        assert_eq!(index::<10, 3>(Vector([10, 2, 2])), None);
        assert_eq!(index::<10, 3>(Vector([9, 9, 9])), Some(999));
        assert_eq!(index::<10, 3>(Vector([0, 0, 0])), Some(0));
        assert_eq!(index::<10, 3>(Vector([-1, 2, 2])), None);
        assert_eq!(index::<10, 3>(Vector([2, -1, 2])), None);
        assert_eq!(index::<10, 3>(Vector([-1, 2, -1])), None);
    }

    #[test]
    fn test_deindex() {
        assert_eq!(deindex::<100, 2>(210), Some(Vector([2, 10])));
        assert_eq!(deindex::<10, 2>(99), Some(Vector([9, 9])));
        assert_eq!(deindex::<10, 2>(0), Some(Vector([0, 0])));

        assert_eq!(deindex::<5, 3>(99), Some(Vector([3, 4, 4])));

        assert_eq!(deindex::<100, 2>(10000), None);
        assert_eq!(deindex::<100, 2>(-1), None);

        assert_eq!(deindex::<100, 3>(1000000), None);
        assert_eq!(deindex::<100, 3>(-1), None);
    }
}

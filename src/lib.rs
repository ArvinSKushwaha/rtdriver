pub mod simulator;
pub mod vector;

pub use simulator::Simulation;
pub use vector::Vector;

pub const DIMS: usize = 2;

const fn stencil<const DIMS: usize>() -> [[Vector<isize, DIMS>; 2]; DIMS] {
    let mut stencil = [[Vector([0; DIMS]); 2]; DIMS];

    let mut i = 0;
    while i < DIMS {
        stencil[i][0].0[i] = 1;
        stencil[i][1].0[i] = -1;

        i += 1;
    }

    stencil
}

pub const STENCIL: [[Vector<isize, DIMS>; 2]; DIMS] = stencil();

pub trait Float: num::Float + bytemuck::Pod {}
impl<T: num::Float + bytemuck::Pod> Float for T {}

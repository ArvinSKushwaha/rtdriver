use rtdriver::Simulation;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let mut sim = Simulation::<f32, 16>::build()
        .stiffness(0.1)
        .origin_stiffness(10.)
        .finish();

    for _ in tqdm::tqdm(0..1_000_000) {
        sim.update(1e-4);
    }
}

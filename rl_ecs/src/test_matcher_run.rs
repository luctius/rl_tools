#![feature(const_generics)]
#![feature(test)]

use rl_ecs::{stores::{Builder, FinishBuilding as _},
             BinStorage, Parent, QueryBuilder, QueryRun, Read, Ref, RefMut, RlEcs, Target, Write};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Position {
    x: usize,
    y: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Velocity {
    x: usize,
    y: usize,
}

const N: usize = 800000;
const N_PER_VEL: usize = 1000;

fn build<T, const SIZE: usize>(ecs: &mut RlEcs<T, { SIZE }>)
    where T: BinStorage, {
    ecs.reserve::<Position>(N);
    ecs.reserve::<Velocity>(N / N_PER_VEL);

    for i in 0..N {
        let e = ecs.create(Position { x: 0, y: 0, });
        if i % N_PER_VEL == 0 {
            ecs.create_and_attach(Velocity { x: i, y: i, }, e).unwrap();
        }
    }
}

fn query_s(vel: &mut Ref<Velocity>, pos: &mut RefMut<Position>) {
    pos.x = std::hint::black_box(std::hint::black_box(pos.x) + std::hint::black_box(vel.x));
    pos.y = std::hint::black_box(std::hint::black_box(pos.y) + std::hint::black_box(vel.y));
}

fn main() {
    let mut ecs = Builder::new().add_component::<Position>().add_component::<Velocity>().finalize();
    build(&mut ecs);

    let mut query = <(Read<Target<Velocity>>, Write<Parent<Position>>)>::build_query();

    for _ in 0..N {
        query.run(&mut ecs, query_s);
        query.run(&mut ecs, |vel, pos| {
                 pos.x = std::hint::black_box(std::hint::black_box(pos.x) + std::hint::black_box(vel.x));
                 pos.y = std::hint::black_box(std::hint::black_box(pos.y) + std::hint::black_box(vel.y));
             });
        // query.run_with_world_mut(&mut ecs, |ecs, (_vel_id, _vel), (pos_id,
        // _pos)| { let _ = ecs.create_and_attach(Velocity { x: 0, y: 0,
        // }, pos_id); });
    }
}

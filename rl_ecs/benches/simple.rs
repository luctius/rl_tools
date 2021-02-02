#![feature(const_generics)]

use criterion::*;

use rl_ecs::{stores::{Builder, FinishBuilding as _},
             BinStorage, EntityRelationType, Parent, Query, QueryBuilder, QueryPurgeAll, QueryPurgeAny, QueryRun,
             Read, RlEcs, Target, Write};

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
const N_PER_VEL: usize = 100;

fn build<T, const SIZE: usize>(ecs: &mut RlEcs<T, { SIZE }>)
    where T: BinStorage, {
    ecs.reserve::<Position>(N);
    ecs.reserve::<Velocity>(N / N_PER_VEL);

    for _ in 0..N - (N / N_PER_VEL) {
        ecs.create(Position { x: 0, y: 0, });
    }
    for i in 0..N_PER_VEL {
        let e = ecs.create(Position { x: 0, y: 0, });
        ecs.create_and_attach(Velocity { x: i, y: i, }, e).unwrap();
    }
}

fn bench_build(b: &mut Criterion) {
    b.bench_function("build", move |b| {
         b.iter(|| {
              let mut ecs = Builder::new().add_component::<Position>().add_component::<Velocity>().finalize();
              build(&mut ecs);
          })
     });
}

fn bench_purge_any(b: &mut Criterion) {
    b.bench_function("purge_any", move |b| {
         b.iter(|| {
              let mut ecs = Builder::new().add_component::<Position>().add_component::<Velocity>().finalize();
              build(&mut ecs);

              <(Read<Target<Velocity>>, Write<Parent<Position>>)>::build_query().purge_any(&mut ecs,
                                                                                    |(_vel_id, _), (pos_id, _)| {
                                                                                        Some(pos_id)
                                                                                    });
              <(Write<Target<Position>>,)>::build_query().purge_all(&mut ecs);
          })
     });
}

fn bench_purge(b: &mut Criterion) {
    b.bench_function("purge", move |b| {
         b.iter(|| {
              let mut ecs = Builder::new().add_component::<Position>().add_component::<Velocity>().finalize();
              build(&mut ecs);

              <(Write<Target<Velocity>>,)>::build_query().purge_all(&mut ecs);
              <(Write<Target<Position>>,)>::build_query().purge_all(&mut ecs);
          })
     });
}

fn bench_purge_once(b: &mut Criterion) {
    b.bench_function("purge_once", move |b| {
         b.iter(|| {
              let mut ecs = Builder::new().add_component::<Position>().add_component::<Velocity>().finalize();
              build(&mut ecs);

              <(Write<Target<Position>>,)>::build_query().purge_all(&mut ecs);
          })
     });
}

fn matcher_run(b: &mut Criterion) {
    let mut ecs = Builder::new().add_component_with_type::<Position>(EntityRelationType::Root)
                                .add_component_with_type::<Velocity>(EntityRelationType::Flag)
                                .finalize();
    build(&mut ecs);

    let mut query = <(Read<Target<Velocity>>, Write<Parent<Position>>)>::build_query();

    b.bench_function("matcher_run", move |b| {
         b.iter(|| {
              query.run(&mut ecs, |vel, pos| {
                       black_box(pos.x += vel.x);
                       black_box(pos.y += vel.y);
                   });
          })
     });
}

criterion_group!(benches, /* bench_build, bench_purge_any, bench_purge_once, bench_purge, */ matcher_run,);
criterion_main!(benches);

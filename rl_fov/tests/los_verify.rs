// Test cases from 'http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#raycode'
use lazy_static::*;

use rl_fov::{utils::TestMap, Fov, FovConfig, Los, VisionShape};

use rl_fov::{
    precalculated_raycasting::{PCRCbuffer, PCRC},
    rpsc::Rpsc,
    shadow_casting::ShadowCasting,
};
// use rl_fov::bevelled_walls::BevelledWalls;
// use rl_fov::diamond_walls::DiamondWalls;

lazy_static! {
    static ref PRC_BUF: PCRCbuffer = PCRCbuffer::generate();
}

fn verify_los<L>(name: &str, mut los: L, map: &TestMap)
    where L: Los + FovConfig, {
    let size = map.size();
    let ppos = map.player_pos().unwrap();

    let mut fov_touched = 0;
    let mut los_touched = 0;
    let mut los_and_fov_touched = 0;

    for y in 0..size.y {
        for x in 0..size.x {
            if ppos == (x, y).into() {
                continue;
            }

            let reach = los.los(ppos, (x, y).into());

            if map.is_touched((x, y).into()) {
                if reach {
                    los_and_fov_touched += 1;
                } else {
                    fov_touched += 1;
                    println!("fov only coord: {},{}", x, y);
                }
            } else if reach {
                los_touched += 1;
                println!("los only coord: {},{}", x, y);
            }
        }
    }

    println!("{}: fov: {}, los: {}, both: {}", name, fov_touched, los_touched, los_and_fov_touched);
}

#[test]
pub fn los_verify() {
    struct Func(Box<dyn Fn(&TestMap) -> TestMap>);

    let mut tests = vec![];
    let fovs = vec![Func(Box::new(|test: &TestMap| {
                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             PCRC { area: map.area(),
                                    buffer: &PRC_BUF,
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.fov(player_pos);

                             let los = PCRC { area: map.area(),
                                              buffer: &PRC_BUF,
                                              callback: TestMap::los_test_func,
                                              cb_type: &mut map.clone(),
                                              radius,
                                              vision: VisionShape::Octagon };

                             verify_los("PCRC", los, &map);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             Rpsc { area: map.area(),
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.fov(player_pos);

                             let los = Rpsc { area: map.area(),
                                              callback: TestMap::los_test_func,
                                              cb_type: &mut map.clone(),
                                              radius,
                                              vision: VisionShape::Octagon };

                             verify_los("Rpsc", los, &map);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             ShadowCasting { area: map.area(),
                                             callback: TestMap::fov_func,
                                             cb_type: &mut map,
                                             radius,
                                             vision: VisionShape::Octagon,
                                             symmetric: true }.fov(player_pos);

                             let los = ShadowCasting { area: map.area(),
                                                       callback: TestMap::los_test_func,
                                                       cb_type: &mut map.clone(),
                                                       radius,
                                                       vision: VisionShape::Octagon,
                                                       symmetric: true };

                             verify_los("ShadowCasting", los, &map);

                             map
                         })),];

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#  ###############  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #      @      #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  #             #  #
#  ###############  #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    for t in tests {
        for f in &fovs {
            f.0(&t).print();
        }
    }
}

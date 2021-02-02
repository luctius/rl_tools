// Test cases from 'http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#raycode'

use lazy_static::*;

use rl_fov::{utils::TestMap, Fov, VisionShape};

use rl_fov::{
    bevelled_walls::BevelledWalls,
    diamond_walls::DiamondWalls,
    precalculated_raycasting::{PCRCbuffer, PCRC},
    rpsc::Rpsc,
    shadow_casting::ShadowCasting,
};

lazy_static! {
    static ref PRC_BUF: PCRCbuffer = PCRCbuffer::generate();
}

pub fn main() {
    struct Func(Box<dyn Fn(&TestMap) -> TestMap>);

    let mut tests = vec![];
    let fovs = vec![Func(Box::new(|test: &TestMap| {
                             println!("PCRC");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             PCRC { area: map.area(),
                                    buffer: &PRC_BUF,
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.fov(player_pos);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("Rpsc");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             Rpsc { area: map.area(),
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.fov(player_pos);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("ShadowCasting");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             ShadowCasting { area: map.area(),
                                             callback: TestMap::fov_func,
                                             cb_type: &mut map,
                                             radius,
                                             symmetric: true,
                                             vision: VisionShape::Octagon }.fov(player_pos);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("DiamondWalls");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             DiamondWalls { area: map.area(),
                                            callback: TestMap::fov_func,
                                            cb_type: &mut map,
                                            radius,
                                            symmetric: true,
                                            vision: VisionShape::Octagon }.fov(player_pos);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("BevelledWalls");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();

                             BevelledWalls { area: map.area(),
                                             callback: TestMap::fov_func,
                                             cb_type: &mut map,
                                             radius,
                                             symmetric: true,
                                             vision: VisionShape::Octagon }.fov(player_pos);

                             map
                         })),];

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#         #         #
#         @         #
#         #         #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#     @             #
#          #        #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################
# @                 #
######### #####     #
#         #   #######
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################
#@                  #
######### #####     #
#         #   #######
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################
#  @                #
######### #####     #
#         #   #######
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#         @         #
#       # # #       #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################
#                   #
##########@##########
#       #  #        #
#      #  #         #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
###################################
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#####       #   #                 #
#@                                #
#####       #   #                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
#                                 #
###################################"
                                    .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#############################################
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
###############       #   #                 #
#@                                          #
###############       #   #                 #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#                                           #
#############################################"
                                              .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
#####################
#             #     #
#            #      #
#           #       #
#          #        #
#         #         #
#        #          #
#       #           #
#      #            #
#    @#             #
#    #              #
#   #               #
#  #                #
# #                 #
##                  #
#                   #
#                   #
#                   #
#                   #
#                   #
#                   #
#####################"
                      .to_string(),
    ),);

    tests.push(TestMap::new(
        "\
############################################################
#                                                          #
# # # #                                        #           #
#  # #                                         #           #
# # # #       ##              #                #           #
#  # #        ##              #                #           #
# # # #                       #       #                    #
#      #            @                          #           #
#       #                  #                               #
#        #                 #   #  #            #           #
#          #               #                   #           #
#           #                                  ### ## #### #
#            #                          # #    #           #
#             #                                #           #
#             ##                      #        #           #
#               ##                             #           #
#                ############### ###############           #
#                                                          #
############################################################"
                                                             .to_string(),
    ),);

    for t in &tests {
        for f in &fovs {
            f.0(&t);
            f.0(&t).print();
        }
    }
}

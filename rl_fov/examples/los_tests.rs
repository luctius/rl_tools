// Test cases from 'http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#raycode'
use lazy_static::*;

use rl_fov::{utils::TestMap, Los, VisionShape};

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

pub fn main() {
    struct Func(Box<dyn Fn(&TestMap) -> TestMap>);

    let mut tests = vec![];
    let fovs = vec![Func(Box::new(|test: &TestMap| {
                             println!("PCRC");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();
                             let dst = map.destination().unwrap();

                             PCRC { area: map.area(),
                                    buffer: &PRC_BUF,
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.los(player_pos, dst);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("Rpsc");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();
                             let dst = map.destination().unwrap();

                             Rpsc { area: map.area(),
                                    callback: TestMap::fov_func,
                                    cb_type: &mut map,
                                    radius,
                                    vision: VisionShape::Octagon }.los(player_pos, dst);

                             map
                         })),
                    Func(Box::new(|test: &TestMap| {
                             println!("ShadowCasting");

                             let mut map = test.clone();
                             let radius = map.size().x as usize;
                             let player_pos = map.player_pos().unwrap();
                             let dst = map.destination().unwrap();

                             ShadowCasting { area: map.area(),
                                             callback: TestMap::fov_func,
                                             cb_type: &mut map,
                                             radius,
                                             symmetric: true,
                                             vision: VisionShape::Octagon }.los(player_pos, dst);

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
#             #x    #
#              #    #
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
#####################
# @               x #
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
#    x              #
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
######### #####    x#
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
#       # # #     x #
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
#       x           #
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
#@                        x       #
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
###############       #   #      x          #
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
# #      x          #
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
#       #                  #              x                #
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

    for t in tests {
        for f in &fovs {
            f.0(&t).print();
        }
    }
}

#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate conveyor;

use conveyor::sentry::{Guard, Range, Access, Action, Sentinel};

use conveyor::sentry;

// describe! region_api {
//     before_each {
//         let region = create_region(0xBA53AD44, 
//                     0x1000,
//                     0,
//                     0,
//                     0,
//                     0,
//                     0,
//                     0);
        
//     }
//     it "region is created" {
//         assert!(region.is_ok());
//     }
//     it "gets info of a region" {
//         let info = get_region(region.id());
//         assert!(region.is_ok());
//     }
//     it "region receives a > 0 id" {
//         assert!(region.unwrap().id() > 0);
//     }
//     it "enables a region" {
//         assert!(enable_region())
//     }
//     it "disables a region" {
//         assert!(get_region())
//     }
//     it "enumerate regions" {
//         for region in regions() {
//             assert!(region.something == true)
//         }
//     }

// }

// describe! region_object {
//     before_each {
//         let region = Region::new();
        
//     }
//     it "has address" {
//         assert!(region.address())
//     }
// }
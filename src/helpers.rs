use core::panic;
use std::{error::Error};
use crate::structs::{WhatsappSource, Source};

use crate::structs::part_request::VehicleDataBuilder;

pub fn whatsapp_reference_exists(tracker_id: &str) -> bool{
    false
}



// pub fn determine_origin<T: Source>(origin_system: &str) -> Box<VehicleDataBuilder<T>> {
//
//     // Transform id to u16
//     let id = origin_system.parse::<u16>().unwrap();
//
//     match id {
//         3 => {
//             Box::new(VehicleDataBuilder::<WhatsappSource>::default()
//                 .build())
//         },
//         1 => {
//             Box::new(VehicleDataBuilder::<WhatsappSource>::default()
//                 .build())
//         }
//         _ => {
//             panic!("Unexpected system id")
//         }
//     }
// }

#[cfg(test)]
mod vin_tests {

    // Passes if vehicles from 2010 and 2039 year is returned correctly
    #[test]
    fn decoded_year_current_rotation() {
        assert!(false);
    }

    // Passes if vehicles from 1980 and 2009 year is returned correctly
    #[test]
    fn decoded_year_old_rotation() {
        assert!(false);
    }
}


#[cfg(test)]
mod request_build {
    use crate::helpers::{determine_origin, is_whatsapp_origin};

    // Passes if passed id return 'WHATSAPP' origin
    #[test]
    fn determine_origin_whatsapp() {
        let whatsapp_origin:u16 = 3;
        assert_eq!(determine_origin(&whatsapp_origin), String::from("WHATSAPP"));
    }

    // Passes if log register is a existing tracker from whatsapp workflow
    #[test]
    fn register_origin_is_whatsapp() {
        assert!(is_whatsapp_origin("1234"));
    }

    // Passes if an error is thrown when trying to create a request with an id that already exists
    #[test]
    fn fails_request_duplicated() {
        assert!(false);
    }

    // Passes if a requestor id phone number matches the log phone number
    #[test]
    fn requestor_matches_log_phone() {
        assert!(false);
    }

    // Passes if a the vehicle brand associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn brand_is_set() {
        assert!(false);
    }

    // Passes if a the vehicle model associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn model_is_set() {
        assert!(false);
    }

    // Passes if a the vehicle vin associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn vin_is_set() {
        assert!(false);
    }
}

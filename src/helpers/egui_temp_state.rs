use std::marker::PhantomData;

use egui::{Id, Ui};

// #[derive(Default)]
pub struct TempState<HeldState> {
    /// A little helper variable that tracks when the state was taken, whether it was written back, and panics if not.
    /// I might move this into another variable to make it configurable, with another struct representing that its fint ot just read.
    // unwritten_data: bool,
    _phantom: PhantomData<HeldState>,
}

// const NO_DATA: & str = "No value present";

impl<T: Clone + Send + Sync + 'static> TempState<T> {
    #[allow(unused)]
    pub fn create_default(ui: &Ui) -> Self
    where
        T: Default,
    {
        Self::create(Default::default(), ui)
    }

    pub fn create(val: T, ui: &Ui) -> Self {
        ui.data_mut(|type_map| {
            assert!(type_map.get_temp::<T>(Id::NULL).is_none(), "There was already duplicate data present. Wrap this type in a unique wrapper to differentiate.");
            type_map.insert_temp(Id::NULL, val);
        });
        Self {
            // unwritten_data: false,
            _phantom: PhantomData,
        }
    }

    // pub fn get_data(ui: &Ui) -> T {
    //     // self.unwritten_data = true;
    //     ui.data(|type_map| type_map.get_temp(Id::NULL).expect(NO_DATA))
    // }

    pub fn peek_data(ui: &Ui) -> Option<T> {
        ui.data(|type_map| type_map.get_temp(Id::NULL))
    }

    pub fn set_or_insert_data(ui: &Ui, val: T) {
        // self.unwritten_data = false;
        ui.data_mut(|type_map| {
            type_map.insert_temp(Id::NULL, val);
        });
    }

    // pub fn set_data(ui: &Ui, val: T) {
    //     // self.unwritten_data = false;
    //     ui.data_mut(|type_map| {
    //         assert!(type_map.get_temp::<T>(Id::NULL).is_some(), "{NO_DATA}");
    //         type_map.insert_temp(Id::NULL, val);
    //     });
    // }

    pub fn finished(ui: &Ui) {
        // self.unwritten_data = false;
        ui.data_mut(|type_map| type_map.remove::<T>(Id::NULL));
    }
}

// impl<T> Drop for TempState<T> {
//     fn drop(&mut self) {
//         assert!(!self.unwritten_data, "Unwritten Data remained.");
//     }
// }

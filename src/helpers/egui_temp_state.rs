use std::marker::PhantomData;

pub struct TempState<'a, T, Del>{
    delegate: &'a Del,
    id: egui::Id,
    _phantom: PhantomData<T>,
}

pub trait TempStateExtDelegatedToDataMethods {
    fn data_mut<R>(&self, writer: impl FnOnce(&mut egui::util::IdTypeMap) -> R) -> R;
    fn data<R>(&self, reader: impl FnOnce(&egui::util::IdTypeMap) -> R) -> R;
    fn temp_state<'a, T>(&'a self) -> TempState<'a, T, Self> where Self: Sized;
}

macro_rules! delegete_for {
    ($type: ty) => {
        impl TempStateExtDelegatedToDataMethods for $type {
            fn data<R>(&self, reader: impl FnOnce(&egui::util::IdTypeMap) -> R) -> R {
                self.data(reader)
            }
            fn data_mut<R>(&self, writer: impl FnOnce(&mut egui::util::IdTypeMap) -> R) -> R {
                self.data_mut(writer)
            }
            fn temp_state<'a, T>(&'a self) -> TempState<'a, T, Self>{
                TempState {
                    delegate: self,
                    id: egui::Id::NULL,
                    _phantom: PhantomData
                }
            }
        }
    };
}

delegete_for!(egui::Ui);
delegete_for!(egui::Context);

impl<'a, T: Clone + Send + Sync + 'static, Del: TempStateExtDelegatedToDataMethods> TempState<'a, T, Del> {
    pub fn with_id(self, id: egui::Id) -> Self {
        Self {
            id,
            ..self
        }
    }
    
    pub fn create(&self, val: T) {
        self.delegate.data_mut(|type_map| {
            assert!(type_map.get_temp::<T>(self.id).is_none(), "There was already duplicate data present. Wrap this type in a unique wrapper to differentiate or provide a unique ID.");
            type_map.insert_temp(self.id, val);
        });
    }
    
    pub fn create_default(&self)
    where
    T: Default,
    {
        self.create(Default::default())
    }
    
    pub fn set_or_create(&self, val: T) {
        self.delegate.data_mut(|type_map| {
            type_map.insert_temp(self.id, val);
        });
    }
    
    pub fn get(&self) -> Option<T> {
        self.delegate.data(|type_map| type_map.get_temp(self.id))
    }
    
    pub fn remove(&self) {
        self.delegate.data_mut(|type_map| type_map.remove::<T>(self.id));
    }
}

// pub trait TempStateExt<T> {
//     fn create(&self, val: T);
//     fn create_default(&self)
//     where
//         T: Default,
//     {
//         self.create(Default::default())
//     }
//     fn set_or_create(&self, val: T);
//     fn get_ref(&self) -> Option<&T>;
//     fn get_mut(&self) -> Option<&mut T>;
//     fn get(&self) -> Option<T>;
//     fn remove(&self);
// }

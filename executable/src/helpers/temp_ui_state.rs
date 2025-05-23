use std::marker::PhantomData;

/// Do NOT use this with resource managers, use [`McmcDemo::local_temp_resources`] instead.
/// This is only for copyable local state (so without eg. references) where its more comfortable to use.
pub struct TempUiState<'a, T, Del> {
    delegate: &'a Del,
    id: egui::Id,
    _phantom: PhantomData<T>,
}

pub trait TempStateDataAccess {
    fn data_mut<R>(&self, writer: impl FnOnce(&mut egui::util::IdTypeMap) -> R) -> R;
    fn data<R>(&self, reader: impl FnOnce(&egui::util::IdTypeMap) -> R) -> R;
    #[must_use]
    fn temp_ui_state<T>(&self) -> TempUiState<'_, T, Self>
    where
        Self: Sized;
}

macro_rules! delegete_for {
    ($type: ty) => {
        impl TempStateDataAccess for $type {
            fn data<R>(&self, reader: impl FnOnce(&egui::util::IdTypeMap) -> R) -> R {
                self.data(reader)
            }
            fn data_mut<R>(&self, writer: impl FnOnce(&mut egui::util::IdTypeMap) -> R) -> R {
                self.data_mut(writer)
            }
            fn temp_ui_state<T>(&self) -> TempUiState<'_, T, Self> {
                TempUiState {
                    delegate: self,
                    id: egui::Id::NULL,
                    _phantom: PhantomData,
                }
            }
        }
    };
}

delegete_for!(egui::Ui);
delegete_for!(egui::Context);

impl<T: Copy + Clone + Send + Sync + 'static, Del: TempStateDataAccess> TempUiState<'_, T, Del> {
    pub const fn with_id(self, id: egui::Id) -> Self {
        Self { id, ..self }
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
        self.create(Default::default());
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
        self.delegate
            .data_mut(|type_map| type_map.remove::<T>(self.id));
    }
}

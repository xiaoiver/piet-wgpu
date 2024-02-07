use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor,
};

pub fn init_composer() -> Composer {
    let mut composer = Composer::default();

    let mut load_composable = |source: &str, file_path: &str| {
        match composer.add_composable_module(ComposableModuleDescriptor {
            source,
            file_path,
            ..Default::default()
        }) {
            Ok(_module) => {
                // println!("{} -> {:#?}", module.name, module)
            }
            Err(e) => {
                println!("? -> {e:#?}")
            }
        }
    };
    composer
}

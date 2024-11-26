use once_cell::sync::OnceCell;

use crate::ExternalConfig;

pub static CONFIG_INSTANCE: OnceCell<ExternalConfig> = OnceCell::new();

pub fn init_config(conf: ExternalConfig) {
    CONFIG_INSTANCE
        .set(conf)
        .expect("Config can only be initialized once");
}

pub fn get_config() -> &'static ExternalConfig {
    CONFIG_INSTANCE
        .get()
        .expect("Config must be initialized before accessing it")
}

// pub fn add_app(appas :Vec<String>)  {
//     let mut config = CONFIG_INSTANCE.get_mut().unwrap();
//     let mut old_apps = config.apps.clone();

//     for app in appas {
//         old_apps.push(',');
//         old_apps.push_str(&app);
//     }

//     println!("{}", old_apps);

//     // 修改 ExternalConfig 中的 apps 属性
//     config.apps = old_apps;

// }


#[cfg(test)]
mod tests {
    use toml_env::Args;

    use crate::conf::singleton;

    // 注意这个惯用法：在 tests 模块中，从外部作用域导入所有名字。
    use super::*;
    #[test]
    fn test_addapp() {

        // let conf :ExternalConfig  = toml_env::initialize(Args::default()).unwrap().unwrap();
        // singleton::init_config(conf);

        // let apps : Vec<String> = vec!["afun".to_string(),"rico".to_string(),"ir6".to_string()];
        // add_app(apps);
        // let config = get_config();
        // println!("{:?}",config.apps)
    }

}
use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Arabic,
    French,
    Chinese,
}

pub struct Localizer {
    translations: HashMap<&'static str, HashMap<Language, &'static str>>,
}

impl Localizer {
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        translations.insert(
            "theme_label",
            HashMap::from([
                (Language::English, "Theme:"),
                (Language::Arabic, "المظهر:"),
                (Language::French, "Thème :"),
                (Language::Chinese, "主题："),
            ]),
        );

        translations.insert(
            "switch_light_mode",
            HashMap::from([
                (Language::English, "Switch to Light Mode"),
                (Language::Arabic, "التحويل إلى الوضع الفاتح"),
                (Language::French, "Passer en mode clair"),
                (Language::Chinese, "切换到浅色模式"),
            ]),
        );

        translations.insert(
            "switch_dark_mode",
            HashMap::from([
                (Language::English, "Switch to Dark Mode"),
                (Language::Arabic, "التحويل إلى الوضع الداكن"),
                (Language::French, "Passer en mode sombre"),
                (Language::Chinese, "切换到深色模式"),
            ]),
        );

        translations.insert(
            "operation_in_progress",
            HashMap::from([
                (Language::English, "Operation in progress..."),
                (Language::Arabic, "العملية جارية..."),
                (Language::French, "Opération en cours..."),
                (Language::Chinese, "操作进行中..."),
            ]),
        );

        translations.insert(
            "connect_to_ssh",
            HashMap::from([
                (Language::English, "Connect to SSH Server"),
                (Language::Arabic, "الاتصال بخادم SSH"),
                (Language::French, "Se connecter au serveur SSH"),
                (Language::Chinese, "连接到SSH服务器"),
            ]),
        );

        translations.insert(
            "saved_connections",
            HashMap::from([
                (Language::English, "Saved Connections:"),
                (Language::Arabic, "الاتصالات المحفوظة:"),
                (Language::French, "Connexions enregistrées :"),
                (Language::Chinese, "已保存的连接："),
            ]),
        );

        translations.insert(
            "no_saved_connections",
            HashMap::from([
                (Language::English, "No saved connections."),
                (Language::Arabic, "لا توجد اتصالات محفوظة."),
                (Language::French, "Aucune connexion enregistrée."),
                (Language::Chinese, "没有已保存的连接。"),
            ]),
        );

        translations.insert(
            "select_connection_combo_label",
            HashMap::from([
                (Language::English, "Select"),
                (Language::Arabic, "اختر"),
                (Language::French, "Sélectionner"),
                (Language::Chinese, "选择"),
            ]),
        );

        translations.insert(
            "choose_a_connection",
            HashMap::from([
                (Language::English, "Choose a connection"),
                (Language::Arabic, "اختر اتصالاً"),
                (Language::French, "Choisissez une connexion"),
                (Language::Chinese, "选择一个连接"),
            ]),
        );

        translations.insert(
            "hostname_label",
            HashMap::from([
                (Language::English, "Hostname:"),
                (Language::Arabic, "اسم المضيف:"),
                (Language::French, "Nom d'hôte :"),
                (Language::Chinese, "主机名："),
            ]),
        );

        translations.insert(
            "username_label",
            HashMap::from([
                (Language::English, "Username:"),
                (Language::Arabic, "اسم المستخدم:"),
                (Language::French, "Nom d'utilisateur :"),
                (Language::Chinese, "用户名："),
            ]),
        );

        translations.insert(
            "password_label",
            HashMap::from([
                (Language::English, "Password:"),
                (Language::Arabic, "كلمة المرور:"),
                (Language::French, "Mot de passe :"),
                (Language::Chinese, "密码："),
            ]),
        );

        translations.insert(
            "port_label",
            HashMap::from([
                (Language::English, "Port:"),
                (Language::Arabic, "المنفذ:"),
                (Language::French, "Port :"),
                (Language::Chinese, "端口："),
            ]),
        );

        translations.insert(
            "save_current_connection",
            HashMap::from([
                (Language::English, "Save Current Connection"),
                (Language::Arabic, "حفظ الاتصال الحالي"),
                (Language::French, "Enregistrer la connexion"),
                (Language::Chinese, "保存当前连接"),
            ]),
        );

        translations.insert(
            "connect_button",
            HashMap::from([
                (Language::English, "Connect"),
                (Language::Arabic, "اتصال"),
                (Language::French, "Se connecter"),
                (Language::Chinese, "连接"),
            ]),
        );

        translations.insert(
            "ssh_file_manager",
            HashMap::from([
                (Language::English, "SSH File Manager"),
                (Language::Arabic, "مدير ملفات SSH"),
                (Language::French, "Gestionnaire de fichiers SSH"),
                (Language::Chinese, "SSH文件管理器"),
            ]),
        );

        translations.insert(
            "current_path_label",
            HashMap::from([
                (Language::English, "Current Path:"),
                (Language::Arabic, "المسار الحالي:"),
                (Language::French, "Chemin actuel :"),
                (Language::Chinese, "当前路径："),
            ]),
        );

        translations.insert(
            "create_directory_label",
            HashMap::from([
                (Language::English, "Create Directory:"),
                (Language::Arabic, "إنشاء مجلد:"),
                (Language::French, "Créer un répertoire :"),
                (Language::Chinese, "创建目录："),
            ]),
        );

        translations.insert(
            "create_file_label",
            HashMap::from([
                (Language::English, "Create File:"),
                (Language::Arabic, "إنشاء ملف:"),
                (Language::French, "Créer un fichier :"),
                (Language::Chinese, "创建文件："),
            ]),
        );

        translations.insert(
            "create_label",
            HashMap::from([
                (Language::English, "Create"),
                (Language::Arabic, "إنشاء"),
                (Language::French, "Créer"),
                (Language::Chinese, "创建"),
            ]),
        );

        translations.insert(
            "directory_name_empty_error",
            HashMap::from([
                (Language::English, "Directory name cannot be empty."),
                (Language::Arabic, "لا يمكن أن يكون اسم الدليل فارغاً."),
                (
                    Language::French,
                    "Le nom du répertoire ne peut pas être vide.",
                ),
                (Language::Chinese, "目录名称不能为空。"),
            ]),
        );

        translations.insert(
            "file_name_empty_error",
            HashMap::from([
                (Language::English, "File name cannot be empty."),
                (Language::Arabic, "لا يمكن أن يكون اسم الملف فارغاً."),
                (Language::French, "Le nom du fichier ne peut pas être vide."),
                (Language::Chinese, "文件名不能为空。"),
            ]),
        );

        translations.insert(
            "up_button",
            HashMap::from([
                (Language::English, "Up"),
                (Language::Arabic, "أعلى"),
                (Language::French, "Haut"),
                (Language::Chinese, "向上"),
            ]),
        );

        translations.insert(
            "home_button",
            HashMap::from([
                (Language::English, "Home"),
                (Language::Arabic, "الرئيسية"),
                (Language::French, "Accueil"),
                (Language::Chinese, "主页"),
            ]),
        );

        translations.insert(
            "disconnect_button",
            HashMap::from([
                (Language::English, "Disconnect"),
                (Language::Arabic, "قطع الاتصال"),
                (Language::French, "Déconnecter"),
                (Language::Chinese, "断开连接"),
            ]),
        );

        translations.insert(
            "download_button",
            HashMap::from([
                (Language::English, "Download"),
                (Language::Arabic, "تنزيل"),
                (Language::French, "Télécharger"),
                (Language::Chinese, "下载"),
            ]),
        );

        translations.insert(
            "delete_button",
            HashMap::from([
                (Language::English, "Delete"),
                (Language::Arabic, "حذف"),
                (Language::French, "Supprimer"),
                (Language::Chinese, "删除"),
            ]),
        );

        translations.insert(
            "modify_button",
            HashMap::from([
                (Language::English, "Modify"),
                (Language::Arabic, "تعديل"),
                (Language::French, "Modifier"),
                (Language::Chinese, "修改"),
            ]),
        );

        translations.insert(
            "rename_button",
            HashMap::from([
                (Language::English, "Rename"),
                (Language::Arabic, "إعادة تسمية"),
                (Language::French, "Renommer"),
                (Language::Chinese, "重命名"),
            ]),
        );

        translations.insert(
            "edit_file_window",
            HashMap::from([
                (Language::English, "Edit File"),
                (Language::Arabic, "تحرير الملف"),
                (Language::French, "Modifier le fichier"),
                (Language::Chinese, "编辑文件"),
            ]),
        );

        translations.insert(
            "editing_label",
            HashMap::from([
                (Language::English, "Editing:"),
                (Language::Arabic, "تحرير:"),
                (Language::French, "Édition :"),
                (Language::Chinese, "编辑中："),
            ]),
        );

        translations.insert(
            "save_button",
            HashMap::from([
                (Language::English, "Save"),
                (Language::Arabic, "حفظ"),
                (Language::French, "Enregistrer"),
                (Language::Chinese, "保存"),
            ]),
        );

        translations.insert(
            "cancel_button",
            HashMap::from([
                (Language::English, "Cancel"),
                (Language::Arabic, "إلغاء"),
                (Language::French, "Annuler"),
                (Language::Chinese, "取消"),
            ]),
        );

        translations.insert(
            "upload_file_button",
            HashMap::from([
                (Language::English, "Upload File"),
                (Language::Arabic, "رفع ملف"),
                (Language::French, "Téléverser un fichier"),
                (Language::Chinese, "上传文件"),
            ]),
        );

        Localizer { translations }
    }

    pub fn t(&self, lang: Language, key: &str) -> &str {
        if let Some(map) = self.translations.get(key) {
            if let Some(value) = map.get(&lang) {
                return value;
            }
        }
        self.translations
            .get(key)
            .and_then(|m| m.get(&Language::English))
            .map_or("MISSING_TRANSLATION", |v| v)
    }
}

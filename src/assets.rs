use macroquad::prelude::*;

// Структура Ассетов
pub struct Assets {
    pub player: Texture2D,
    pub enemy: Texture2D,
    pub phone: Texture2D,
    pub weapons: Texture2D,
    pub cursor: Texture2D,
    pub font: Font,
}

impl Assets {
    pub async fn load() -> Self {
        // Загрузка текстур
        let player_bytes = include_bytes!("../assets/player_tileset.png");
        let phone_bytes = include_bytes!("../assets/phone.png");
        let enemy_bytes = include_bytes!("../assets/enemy_mechanic.png");
        let weapons_bytes = include_bytes!("../assets/weapons.png");
        let cursor_bytes = include_bytes!("../assets/cursor.png");

        let player = Texture2D::from_file_with_format(player_bytes, None);
        let phone = Texture2D::from_file_with_format(phone_bytes, None);
        let enemy = Texture2D::from_file_with_format(enemy_bytes, None);
        let weapons = Texture2D::from_file_with_format(weapons_bytes, None);
        let cursor = Texture2D::from_file_with_format(cursor_bytes, None);

        next_frame().await;

        let font_bytes = include_bytes!("../assets/zinux.ttf");
        let font = load_ttf_font_from_bytes(font_bytes).unwrap();

        // Отключение размытия для пиксель арта
        player.set_filter(FilterMode::Nearest);
        phone.set_filter(FilterMode::Nearest);
        enemy.set_filter(FilterMode::Nearest);
        weapons.set_filter(FilterMode::Nearest);
        cursor.set_filter(FilterMode::Nearest);

        Self {
            player,
            enemy,
            phone,
            weapons,
            cursor,
            font,
        }
    }
}

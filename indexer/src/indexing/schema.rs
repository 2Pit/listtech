use tantivy::schema::*;

pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // Уникальный идентификатор товара
    schema_builder.add_text_field("asin", STRING | STORED);

    // Название товара
    schema_builder.add_text_field("title", TEXT | STORED);

    // Описание товара (массив строк, объединяется)
    schema_builder.add_text_field("description", TEXT | STORED);

    // Основные характеристики (bullet points, массив строк)
    schema_builder.add_text_field("feature", TEXT | STORED);

    // Цена товара в долларах (если распарсилась)
    schema_builder.add_f64_field("price", FAST | STORED);

    // Основная категория товара (текст, может быть многословной)
    schema_builder.add_text_field("main_cat", TEXT | STORED);

    // Название бренда как фасет (для фильтрации)
    schema_builder.add_facet_field("brand", FacetOptions::default().set_stored());

    // Название бренда как строка (для отображения в выдаче)
    schema_builder.add_text_field("brand_string", STRING | STORED);

    // Полный путь категории как фасет (иерархический)
    schema_builder.add_facet_field("category", FacetOptions::default().set_stored());

    // Позиции в рейтингах (multi-valued): 3092, 5010 и т.п.
    schema_builder.add_i64_field("rank_position", FAST | STORED);

    // Категории, в которых товар занимает позицию (соответствуют по индексу с rank_position)
    schema_builder.add_facet_field("rank_facet", FacetOptions::default().set_stored());

    // Список ASIN'ов товаров, которые также покупали
    schema_builder.add_text_field("also_buy", STRING | STORED);

    // Список ASIN'ов товаров, которые также просматривали
    schema_builder.add_text_field("also_view", STRING | STORED);

    // Временная метка создания товара (в миллисекундах, Unix timestamp)
    schema_builder.add_i64_field("timestamp_creation_ms", FAST | STORED);

    // URL основного изображения
    schema_builder.add_text_field("image_url", STRING | STORED);

    // URL изображения в высоком разрешении
    schema_builder.add_text_field("image_url_high_res", STRING | STORED);

    // Первая таблица тех. параметров (как строка)
    schema_builder.add_text_field("tech1", STRING | STORED);

    // Вторая таблица тех. параметров (как строка)
    schema_builder.add_text_field("tech2", STRING | STORED);

    // Схожий товар (обычно один ASIN)
    schema_builder.add_text_field("similar_item", STRING | STORED);

    schema_builder.build()
}

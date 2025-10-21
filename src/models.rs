// use diesel::prelude::*;

// #[derive(Queryable, Selectable, Insertable, Debug, Clone)]
// #[diesel(table_name = crate::schema::novels)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct Novel {
//     id: Option<i32>,
//     title: String,
// }

// impl Novel {
//     pub fn new(title: String) -> Novel {
//         Novel { id: None, title }
//     }

//     pub fn id(&self) -> &Option<i32> {
//         &self.id
//     }

//     pub fn title(&self) -> &String {
//         &self.title
//     }
// }

// #[derive(Queryable, Selectable, Insertable, Debug, Clone)]
// #[diesel(table_name = crate::schema::novel_chapters)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct NovelChapter {
//     pub id: Option<i32>,
//     pub novel_id: i32,
//     pub enumeration: f32,
//     pub title: Option<String>,
//     pub magnet_link: String,
//     pub content_path: Option<String>,
//     pub is_read: bool,
// }

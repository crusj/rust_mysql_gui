use std::fs::OpenOptions;
use std::io::Write;

use iced::widget::{button, column, container, scrollable, text, Rule};
use iced::Length;
use iced::{theme, Element, Sandbox, Settings};
use mysql::prelude::*;
use mysql::*;
const WINDOW_WIDTH: u16 = 1600;
const WINDOW_HEIGHT: u16 = 900;

fn main() -> iced::Result {
    Table::run(Settings {
        default_font: Some(include_bytes!("/home/jianglong/Downloads/wryh/wryh.ttf")),
        window: iced::window::Settings {
            size: (1600, 900),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
#[derive(Debug, Clone)]
pub enum Message {
    NextPage,
    PrePage,
    Table(String),
}

#[derive(Default)]
struct Table {
    page: i32,
    table_name: String,
    columns: Vec<TableColumn>,
}

struct TableColumn {
    column_name: String,
    data_type: String,
}

impl Sandbox for Table {
    type Message = Message;
    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("MYSQL")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::NextPage => {
                self.page += 1;
            }
            Message::PrePage => {
                if self.page >= 1 {
                    self.page -= 1;
                }
            }
            Message::Table(data) => {
                self.table_name = data;
                let db = DB::new();
                let mut conn = db.get_conn().unwrap();
                self.get_columns(&mut conn);
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let db = DB::new();
        let mut conn = db.get_conn().unwrap();

        let width = 40;
        let mut header_row = iced::widget::Row::new();
        for column in self.columns.iter() {
            write_into_file(self.columns.len().to_string().as_str());
            write_into_file(&column.column_name);
            header_row = header_row.push(text(column.column_name.clone()).width(width));
        }

        // let mut content = column!().width(Length::Fill).padding(10);
        let pre_button = button("上一页").on_press(Message::PrePage);
        let next_button = button("下一页").on_press(Message::NextPage);
        let buttons = iced::widget::row!(pre_button, next_button);

        container(iced::widget::row![
            scrollable(self.left(&mut conn)),
            Rule::vertical(10),
            column![
                buttons,
                text(&self.table_name),
                header_row,
                Rule::horizontal(10),
                // scrollable(content),
            ]
            .align_items(iced::Alignment::Start)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

struct DB {
    pool: Pool,
}

impl DB {
    fn new() -> Self {
        DB {
            pool: Pool::new("mysql://root:cloud&root@192.168.2.121:13306/cloud_dev").unwrap(),
        }
    }

    fn get_conn(&self) -> Result<PooledConn> {
        self.pool.get_conn()
    }
}

impl Table {
    fn query(conn: &mut PooledConn) -> anyhow::Result<Vec<String>> {
        let mut fields = Vec::new();

        conn.query_iter(
            "SELECT table_name  FROM information_schema.TABLES WHERE table_schema = 'cloud'",
        )
        .unwrap()
        .for_each(|row| {
            let r: (Value, Value, Value, Value, Value, Value) = from_row(row.unwrap());
            if let Value::Bytes(field) = r.0 {
                fields.push(String::from_utf8(field).unwrap());
            }
        });

        Ok(fields)
    }

    fn left<'a>(&self, conn: &mut PooledConn) -> iced::widget::Column<'a, Message> {
        let width = WINDOW_WIDTH / 5;
        let mut column = iced::widget::Column::new();
        if let Ok(tables) = self.tables(conn) {
            for each in tables {
                column = column.push(iced::widget::row![button(
                    text(each.clone().as_str()).size(20.0)
                )
                .style(theme::Button::Primary)
                .on_press(Message::Table(each.clone())),]);
            }

            column.width(Length::from(width))
        } else {
            column
        }
    }

    fn tables(&self, conn: &mut PooledConn) -> anyhow::Result<Vec<String>> {
        let mut tables = Vec::new();
        let sql = "SELECT table_name  FROM information_schema.TABLES WHERE table_schema = 'cloud'";

        conn.query_iter(sql).unwrap().for_each(|row| {
            let table_name = TableName::parse_row(&mut row.unwrap());
            tables.push(table_name.table_name);
        });

        Ok(tables)
    }
    // 获取表的字段名
    fn get_columns(&mut self, conn: &mut PooledConn) {
        let mut columns = Vec::new();
        let sql = format!(
            "select column_name,data_type from information_schema.COLUMNS where table_name = '{}' ",
            self.table_name
        );

        conn.query_iter(sql).unwrap().for_each(|row| {
            let mut row = row.unwrap();
            columns.push(TableColumn {
                column_name: row.take("column_name").unwrap(),
                data_type: row.take("data_type").unwrap(),
            });
        });

        write_into_file("\n");
        write_into_file(columns.len().to_string().as_str());
        write_into_file("\n");
        self.columns = columns;
    }
}
struct TableName {
    table_name: String,
}

impl TableName {
    fn parse_row(row: &mut Row) -> Self {
        TableName {
            table_name: row.take("table_name").unwrap(),
        }
    }
}

fn write_into_file(log: &str) {
    let mut file = OpenOptions::new().append(true).open("log.txt").unwrap();
    file.write(log.as_bytes());
}

use iced::widget::{button, column, container, scrollable, text, Rule};
use iced::{theme, Color, Element, Length, Sandbox, Settings};
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
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let db = DB::new();
        let mut conn = db.get_conn().unwrap();

        let width = (WINDOW_WIDTH - WINDOW_WIDTH / 5) / 5;
        let header_row = iced::widget::Row::new()
            .padding(10)
            .push(text("id").width(width))
            .push(text("name").width(width))
            .push(text("robot_code").width(width))
            .push(text("body_code").width(width))
            .push(text("company_id").width(width));

        let mut content = column!().width(Length::Fill).padding(10);
        if let Ok(data) = self.data(&mut conn) {
            for each in data {
                content = content.push(
                    iced::widget::Row::new()
                        .padding(10)
                        .push(text(each.id).width(width))
                        .push(text(each.name).width(width))
                        .push(text(each.robot_code).width(width))
                        .push(text(each.body_code).width(width))
                        .push(text(each.company_id).width(width)),
                );
            }
        }

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
                scrollable(content),
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
                column = column.push(
                    button(text(each.clone()))
                        .style(theme::Button::Text)
                        .on_press(Message::Table(each.clone())),
                );
            }
            column.width(Length::from(width)).padding(10)
        } else {
            column
        }
    }

    fn data(&self, conn: &mut PooledConn) -> anyhow::Result<Vec<Robot>> {
        let mut data = Vec::new();
        let mut from = self.page * 20;
        if from < 0 {
            from = 0
        }

        let sql = format!(
            "select id, name, robot_code, elbow_number,body_code,company_id from robot limit {},50",
            from
        );
        print!("{}", &sql);
        conn.query_iter(sql).unwrap().for_each(|row| {
            let robot = Robot::parse_row(&mut row.unwrap());
            data.push(robot);
        });

        Ok(data)
    }

    fn tables(&self, conn: &mut PooledConn) -> Result<Vec<String>> {
        let mut tables = Vec::new();

        conn.query_iter(
            "SELECT table_name  FROM information_schema.TABLES WHERE table_schema = 'cloud'",
        )
        .unwrap()
        .for_each(|row| {
            let table_name = TableName::parse_row(&mut row.unwrap());
            tables.push(table_name.table_name);
        });

        Ok(tables)
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

struct Robot {
    id: i32,
    name: String,
    robot_code: String,
    body_code: String,
    company_id: i32,
}

impl Robot {
    fn parse_row(row: &mut Row) -> Self {
        Robot {
            id: row.take("id").unwrap(),
            name: row.take("name").unwrap(),
            robot_code: row.take("robot_code").unwrap(),
            body_code: row.take("body_code").unwrap(),
            company_id: row.take("company_id").unwrap(),
        }
    }
}

use actix_web::{get, App, HttpResponse, HttpServer, Responder, post};
use rusqlite::Connection;
use serde::{Deserialize, Serialize}; // Import Serialize trait
use rusqlite::Result as SqliteResult;

/*
*
* TODO: Inserting data incorrectly - need to fix
*
*/

const DB_PATH: &str = "../rust_prac03/northwind.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: String,
    pub company_name: String,
    pub contact_name: String,
}

#[derive(Serialize, Deserialize, Debug)] // Derive Serialize trait
struct User {
    id: u32,
    name: String,
    email: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProductTotalSales {
    product_id: i32,
    product_name: String,
    unit_price: f64,
    quantity: i32,
    units_on_order: i32,
    total_sales: f64,
}
#[derive(Debug, Serialize, Deserialize)]
struct Order {
    order_id: i32,
    customer_id: String,
    employee_id: i32,
    order_date: Option<String>,
    required_date: Option<String>,
    shipped_date: Option<String>,
    ship_via: i32,
    freight: f64,
    ship_name: Option<String>,
    ship_address: Option<String>,
    ship_city: Option<String>,
    ship_region: Option<String>,
    ship_postal_code: Option<String>,
    ship_country: Option<String>,
}

impl ProductTotalSales {
    fn fetch_prod_total(conn: &rusqlite::Connection) -> SqliteResult<Vec<Self>> {
        let query_string = r#"
            SELECT 
                Products.ProductID, 
                Products.ProductName, 
                OrderDetails.UnitPrice,
                OrderDetails.Quantity, 
                Products.UnitsOnOrder,
                Products.UnitsOnOrder * OrderDetails.UnitPrice AS TotalSales 
            FROM Products
            JOIN "Order Details" as OrderDetails ON Products.ProductID = OrderDetails.ProductID
            GROUP BY Products.ProductID, Products.ProductName
            ORDER BY TotalSales DESC;
        "#;
        let mut query = conn.prepare(query_string)?;
        let prod_total_iter = query.query_map([], |row| {
            Ok(ProductTotalSales {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                unit_price: row.get(2)?,
                quantity: row.get(3)?,
                units_on_order: row.get(4)?,
                total_sales: row.get(5)?,
            })
        })?;

        prod_total_iter.collect()
    }
}

impl Order {
    // A method to fetch orders from the database
    fn fetch_orders(conn: &rusqlite::Connection) -> SqliteResult<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM orders ORDER BY OrderID DESC")?;
        let order_iter = stmt.query_map([], |row| {
            Ok(Order {
                order_id: row.get(0)?,
                customer_id: row.get(1)?,
                employee_id: row.get(2)?,
                order_date: row.get(3)?,
                required_date: row.get(4)?,
                shipped_date: row.get(5)?,
                ship_via: row.get(6)?,
                freight: row.get(7)?,
                ship_name: row.get(8)?,
                ship_address: row.get(9)?,
                ship_city: row.get(10)?,
                ship_region: row.get(11)?,
                ship_postal_code: row.get(12)?,
                ship_country: row.get(13)?,
            })
        })?;
        println!("Fetched orders!");
        order_iter.collect()
    }

    fn insert_order(conn: &mut Connection, order: &Order) -> SqliteResult<()> {
        let query_string = r#"
            INSERT INTO orders (OrderID, CustomerID, EmployeeID, OrderDate, RequiredDate, ShippedDate, ShipVia, Freight, ShipName, ShipAddress, ShipCity, ShipRegion, ShipPostalCode, ShipCountry) VALUES (:order_id, :customer_id, :employee_id, :order_date, :required_date, :shipped_date, :ship_via, :freight, :ship_name, :ship_address, :ship_city, :ship_region, :ship_postal_code, :ship_country);
        "#;

        let stmt = conn.transaction()?;

        let params: Vec<(&str, &dyn rusqlite::types::ToSql)> = vec![
            (":order_id", &order.order_id),
            (":customer_id", &order.customer_id),
            (":employee_id", &order.employee_id),
            (":order_date", &order.order_date),
            (":required_date", &order.required_date),
            (":shipped_date", &order.shipped_date),
            (":ship_via", &order.ship_via),
            (":freight", &order.freight),
            (":ship_name", &order.ship_name),
            (":ship_address", &order.ship_address),
            (":ship_city", &order.ship_city),
            (":ship_region", &order.ship_region),
            (":ship_postal_code", &order.ship_postal_code),
            (":ship_country", &order.ship_country),
        ];

        stmt.execute(query_string, &*params)?;
        stmt.commit()?;

        println!("Inserted new order!");
        Ok(())
    }
}

#[get("/users")]
async fn get_users() -> impl Responder {
    // Make the request to the API
    let request_url = format!("https://jsonplaceholder.typicode.com/users");
    let response = reqwest::get(&request_url).await;
    
    // Check if the request was successful
    match response {
        Ok(response) => {
            match response.json::<Vec<User>>().await {
                Ok(users) => HttpResponse::Ok().json(users),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/orders")]
async fn get_orders() -> impl Responder {
    // Connect to SQLite database
    let conn = Connection::open(DB_PATH).unwrap();

    match Order::fetch_orders(&conn) {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/orders")]
async fn insert_order() -> impl Responder {
    let mut conn = Connection::open(DB_PATH).unwrap();

    let order = Order {
        order_id: 26533,
        customer_id: "RATTC".to_string(),
        employee_id: 1,
        order_date: Some("1998-05-06".to_string()),
        required_date: Some("1998-06-03".to_string()),
        shipped_date: Some("1998-05-08".to_string()),
        ship_via: 2,
        freight: 8.53,
        ship_name: Some("Rattlesnake Canyon Grocery".to_string()),
        ship_address: Some("2817 Milton Dr.".to_string()),
        ship_city: Some("Albuquerque".to_string()),
        ship_region: Some("NM".to_string()),
        ship_postal_code: Some("87110".to_string()),
        ship_country: Some("USA".to_string()),
    };
    
    match Order::insert_order(&mut conn, &order) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/product_total_sales")]
async fn get_prod_total_sales() -> impl Responder {
    let conn = Connection::open(DB_PATH).unwrap();

    match ProductTotalSales::fetch_prod_total(&conn) {
        Ok(prod_total_sales) => HttpResponse::Ok().json(prod_total_sales),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_orders)
            .service(get_prod_total_sales)
            .service(get_users)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

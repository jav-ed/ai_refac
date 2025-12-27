use std::path::Path;
use super::utils::create_file;

pub fn generate(root: &Path) -> std::io::Result<()> {
    let base = root.join("python");
    println!("Generating Complex Python project (E-Commerce Domain)...");

    // Init files
    create_file(&base, "lib/__init__.py", "")?;
    create_file(&base, "lib/models/__init__.py", "")?;
    create_file(&base, "lib/services/__init__.py", "")?;
    create_file(&base, "lib/db/__init__.py", "")?;

    // 1. Config (File 1)
    create_file(&base, "config.py", r#"
DB_HOST = "localhost"
DB_PORT = 5432
MAX_RETRIES = 3
TIMEOUT_SECONDS = 30.0
ENABLE_LOGGING = True
API_KEY = "secret_key_123"
"#)?;

    // 2. Utils (File 2)
    create_file(&base, "lib/utils.py", r#"
import time

def generate_id(prefix: str) -> str:
    timestamp = int(time.time() * 1000)
    return f"{prefix}_{timestamp}"

def validate_price(price: float) -> bool:
    return price >= 0.0
"#)?;

    // 3. Models (File 3, 4)
    create_file(&base, "lib/models/product.py", r#"
from dataclasses import dataclass

@dataclass
class Product:
    id: str
    name: str
    sku: str
    price: float
    stock_quantity: int
    is_active: bool
"#)?;

    create_file(&base, "lib/models/order.py", r#"
from dataclasses import dataclass
from typing import List
from .product import Product

@dataclass
class OrderItem:
    product: Product
    quantity: int

@dataclass
class Order:
    id: str
    items: List[OrderItem]
    total_amount: float
    customer_email: str
    status: str
"#)?;

    // 4. DB Layer (File 5)
    create_file(&base, "lib/db/database.py", r#"
from config import DB_HOST, DB_PORT

class DatabaseConnection:
    def __init__(self):
        self.host = DB_HOST
        self.port = DB_PORT
        self.connected = False

    def connect(self):
        print(f"Connecting to {self.host}:{self.port}...")
        self.connected = True
    
    def disconnect(self):
        self.connected = False
"#)?;

    // 5. Services (File 6)
    create_file(&base, "lib/services/order_service.py", r#"
from lib.models.order import Order, OrderItem
from lib.models.product import Product
from lib.db.database import DatabaseConnection
from lib.utils import generate_id

class OrderService:
    def __init__(self, db: DatabaseConnection):
        self.db = db

    def create_order(self, product: Product, qty: int, email: str) -> Order:
        if not self.db.connected:
            self.db.connect()
        
        # Calculate total
        total = product.price * qty
        order_id = generate_id("ORD")
        
        item = OrderItem(product=product, quantity=qty)
        order = Order(
            id=order_id,
            items=[item],
            total_amount=total,
            customer_email=email,
            status="CREATED"
        )
        return order
"#)?;

    // 6. Main (File 7)
    create_file(&base, "main.py", r#"
from lib.models.product import Product
from lib.services.order_service import OrderService
from lib.db.database import DatabaseConnection
from config import ENABLE_LOGGING

def main():
    if ENABLE_LOGGING:
        print("System starting...")

    # Setup
    db = DatabaseConnection()
    service = OrderService(db)

    # Data
    phone = Product(
        id="prod_1",
        name="Smartphone X",
        sku="SPX-2024",
        price=999.99,
        stock_quantity=50,
        is_active=True
    )

    # Action
    order = service.create_order(phone, 2, "cust@example.com")
    print(f"Order created: {order.id} for ${order.total_amount}")

if __name__ == "__main__":
    main()
"#)?;

    Ok(())
}

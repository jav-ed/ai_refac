# Absolute + aliased import — alias must survive the rewrite.
from myapp.utils.formatters import format_date as fd
from myapp.utils.formatters import format_currency
from myapp.models.base import Base


class Record(Base):
    def __init__(self, amount: float, ts: int) -> None:
        self.amount = amount
        self.ts = ts

    def display(self) -> str:
        return f"{fd(self.ts)}: {format_currency(self.amount)}"

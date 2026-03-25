# Indirect import via __init__ re-export — Rope does NOT update this.
# from ..utils resolves through utils/__init__.py, not directly to formatters.py.
# After the move this still works because utils/__init__.py is updated to re-export
# from the new location — but the text of this file is UNCHANGED by Rope.
from ..utils import format_date, format_currency


class Order:
    def __init__(self, amount: float, ts: int) -> None:
        self.amount = amount
        self.ts = ts

    def summary(self) -> str:
        return f"{format_date(self.ts)} {format_currency(self.amount)}"

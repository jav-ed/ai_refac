# Indirect import via __init__ re-export — Rope does NOT update this.
# Text stays as-is; still works at runtime because utils/__init__ is updated.
from myapp.utils import format_currency, format_date


def process(amount: float, ts: int) -> str:
    return f"{format_date(ts)}: {format_currency(amount)}"

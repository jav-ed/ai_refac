# Relative sibling import — Rope rewrites to absolute after move.
from .formatters import format_currency


def is_valid_amount(amount: float) -> bool:
    return amount > 0


def describe_amount(amount: float) -> str:
    return format_currency(amount) if is_valid_amount(amount) else "invalid"

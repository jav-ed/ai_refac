# MOVE TARGET: myapp/utils/formatters.py -> myapp/core/formatters.py


def format_date(ts: int) -> str:
    return str(ts)


def format_currency(amount: float) -> str:
    return f"${amount:.2f}"


def format_percent(ratio: float) -> str:
    return f"{ratio * 100:.1f}%"

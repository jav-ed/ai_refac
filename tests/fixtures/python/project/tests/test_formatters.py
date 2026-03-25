# Test file — exercises absolute import from a non-package directory.
# Rope should update this the same as any other file.
from myapp.utils.formatters import format_date, format_currency, format_percent


def test_format_date() -> None:
    assert format_date(0) == "0"


def test_format_currency() -> None:
    assert format_currency(9.99) == "$9.99"


def test_format_percent() -> None:
    assert format_percent(0.5) == "50.0%"

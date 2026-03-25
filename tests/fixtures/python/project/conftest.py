# Project-root conftest — exercises import from a file outside the main package.
from myapp.utils.formatters import format_currency


def pytest_configure(config: object) -> None:
    _ = format_currency(0.0)

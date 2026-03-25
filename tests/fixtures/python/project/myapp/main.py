# Absolute multi-name import — verifies Rope rewrites a from-import with multiple names.
from myapp.utils.formatters import format_date, format_currency
from myapp.models import Record


def run() -> None:
    r = Record(amount=9.99, ts=0)
    print(format_date(0))
    print(format_currency(r.amount))

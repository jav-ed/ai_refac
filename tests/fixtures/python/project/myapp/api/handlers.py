# Module-level aliased import — `import X as Y` form.
import myapp.utils.formatters as fmt


def handle(amount: float, ts: int) -> str:
    return f"{fmt.format_date(ts)}: {fmt.format_currency(amount)}"

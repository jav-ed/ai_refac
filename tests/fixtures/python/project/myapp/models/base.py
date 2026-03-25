# Control file — no dependency on formatters. Used as base class by record.py.


class Base:
    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}>"

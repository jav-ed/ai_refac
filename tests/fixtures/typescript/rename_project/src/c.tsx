import { Counter } from "./lib/util";
export function Widget(props: { counter: Counter; label: string }) {
    const { label } = props;
    return <div title={label}>{props.counter.count}</div>;
}

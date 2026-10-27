// React server-side rendering on yavashark.
//
//   yavashark app.tsx > out.html

import React, { renderToString } from "./vendor/react.js";

interface Todo {
    title: string;
    done: boolean;
}

const todos: Todo[] = [
    { title: "Write a JavaScript engine", done: true },
    { title: "Render React with it", done: true },
    { title: "Pass all of test262", done: false },
];

const Theme = React.createContext({ accent: "black" });

const TodoItem = ({ todo }: { todo: Todo }) => {
    const { accent } = React.useContext(Theme);

    return (
        <li style={{ color: todo.done ? "gray" : accent }}>
            {todo.done ? <s>{todo.title}</s> : todo.title}
        </li>
    );
};

const TodoList = ({ items }: { items: Todo[] }) => {
    const left = items.filter((t) => !t.done).length;

    return (
        <section>
            <ul>
                {items.map((todo) => (
                    <TodoItem key={todo.title} todo={todo} />
                ))}
            </ul>
            <p>{left === 0 ? "All done!" : `${left} left to do.`}</p>
        </section>
    );
};

const App = ({ name }: { name: string }) => (
    <Theme.Provider value={{ accent: "#f97316" }}>
        <html lang="en">
            <head>
                <meta charSet="utf-8" />
                <title>{`${name} + React`}</title>
            </head>
            <body>
                <h1>Hello from {name}!</h1>
                <TodoList items={todos} />
            </body>
        </html>
    </Theme.Provider>
);

console.log("<!DOCTYPE html>" + renderToString(<App name="yavashark" />));

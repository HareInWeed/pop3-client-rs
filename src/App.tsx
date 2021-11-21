import React, { FC, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

const App: FC = () => {
  const [count, setCount] = useState(0);

  console.log(window);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div>{window.location.href}</div>
      {count}
      <button
        onClick={async () => {
          await invoke("increase");
          setCount(await invoke("get_counter"));
        }}
      >
        +1
      </button>
      <button
        onClick={async () => {
          await invoke("decrease");
          setCount(await invoke("get_counter"));
        }}
      >
        -1
      </button>
    </div>
  );
};

export default App;

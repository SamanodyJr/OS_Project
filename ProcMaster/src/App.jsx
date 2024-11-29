import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import SysInfo from "./sysinfo"; // Import the SysInfo component

function App() {
  return (
    <main className="container">
      {/* Include the SysInfo component */}
      <SysInfo />
    </main>
  );
}

export default App;

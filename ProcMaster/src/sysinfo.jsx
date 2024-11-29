import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";

const SysInfo = () => {
  const [cpuUsages, setCpuUsages] = useState([]);

  useEffect(() => {
    // Fetch CPU usage data when the component mounts
    fetchCpuUsages();
  }, []);

  const fetchCpuUsages = async () => {
    try {
      const data = await invoke("cpu_resultt");
      console.log("Fetched CPU usage data:", data); // Debugging log
      setCpuUsages(data);
    } catch (error) {
      console.error("Error fetching CPU usage data:", error);
    }
  };

  return (
    <div>
      <h1>CPU Usage</h1>
      <div>
        {cpuUsages.length === 0 ? (
          <p>Loading...</p>
        ) : (
          cpuUsages.map((cpu, index) => (
            <div key={index}>
              <h2>Core {cpu.core_number}</h2>
              <p>Usage: {cpu.cpu_usage.toFixed(1)}%</p>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default SysInfo;

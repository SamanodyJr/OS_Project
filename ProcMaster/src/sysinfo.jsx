import { invoke } from "@tauri-apps/api/core";
import React, { useEffect, useState } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell,
} from "recharts";

const SysInfo = () => {
  const [cpuUsages, setCpuUsages] = useState([]);

  useEffect(() => {
    console.log("Component mounted");
    // Fetch CPU usage data when the component mounts
    fetchCpuUsages();
  }, []);

  const fetchCpuUsages = async () => {
    try {
      console.log("Fetching CPU usage data...");
      const data = await invoke("cpu_resultt");
      console.log("Fetched CPU usage data:", data); // Debugging log
      setCpuUsages(data);
    } catch (error) {
      console.error("Error fetching CPU usage data:", error);
    }
  };

  const getBarColor = (value) => {
    if (value < 20) return "yellow";
    if (value < 40) return "orange";
    return "red";
  };

  return (
    <div>
      <h1>CPU Usage</h1>
      <div>
        {cpuUsages.length === 0 ? (
          <p>Loading...</p>
        ) : (
          <ResponsiveContainer width="100%" height={400}>
            <BarChart
              data={cpuUsages}
              margin={{
                top: 5,
                right: 30,
                left: 20,
                bottom: 5,
              }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="core_number" />
              <YAxis domain={[0, 100]} tickFormatter={(tick) => `${tick}%`} />
              <Tooltip formatter={(value) => `${value}%`} />
              <Legend />
              <Bar dataKey="cpu_usage">
                {cpuUsages.map((entry, index) => (
                  <Cell
                    key={`cell-${index}`}
                    fill={getBarColor(entry.cpu_usage)}
                  />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
};

export default SysInfo;
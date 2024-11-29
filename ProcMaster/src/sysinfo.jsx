import { invoke } from "@tauri-apps/api/core";
import React, { useEffect, useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
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

  return (
    <div>
      <h1>CPU Usage</h1>
      <div>
        {cpuUsages.length === 0 ? (
          <p>Loading...</p>
        ) : (
          <ResponsiveContainer width="100%" height={400}>
            <LineChart
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
              <YAxis />
              <Tooltip />
              <Legend />
              <Line
                type="monotone"
                dataKey="cpu_usage"
                stroke="#8884d8"
                activeDot={{ r: 8 }}
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
};

export default SysInfo;

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
  LineChart,
  Line,
  PieChart,
  Pie,
} from "recharts";

const SysInfo = () => {
  const [cpuUsages, setCpuUsages] = useState([]);
  const [memoryUsage, setMemoryUsage] = useState(null);

  useEffect(() => {
    console.log("Component mounted");
    // Fetch CPU usage data when the component mounts
    fetchCpuUsages();
    fetchMemoryUsage();

    // Set up an interval to fetch the data every 5 seconds
    const interval = setInterval(() => {
      fetchCpuUsages();
      fetchMemoryUsage();
    }, 5000);

    // Clean up the interval on component unmount
    return () => clearInterval(interval);
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

  const fetchMemoryUsage = async () => {
    try {
      console.log("Fetching memory usage data...");
      const data = await invoke("Mem_Usage");
      console.log("Fetched memory usage data:", data); // Debugging log
      setMemoryUsage(data);
    } catch (error) {
      console.error("Error fetching memory usage data:", error);
    }
  };

  const getBarColor = (value) => {
    if (value < 20) return "yellow";
    if (value < 40) return "orange";
    return "red";
  };
  const memoryData = memoryUsage
    ? [
        { name: "Used Memory", value: memoryUsage.used },
        { name: "Free Memory", value: memoryUsage.free },
      ]
    : [];

  const swapData = memoryUsage
    ? [
        { name: "Used Swap", value: memoryUsage.used_swap },
        { name: "Free Swap", value: memoryUsage.free_swap },
      ]
    : [];

  const COLORS = ["#8884d8", "#82ca9d"];

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
      <h1>Memory and Swap Usage</h1>
      <div
        style={{
          display: "flex",
          justifyContent: "space-around",
          alignItems: "center",
        }}
      >
        {memoryUsage === null ? (
          <p>Loading...</p>
        ) : (
          <>
            <ResponsiveContainer width="45%" height={400}>
              <PieChart>
                <Pie
                  data={memoryData}
                  cx="50%"
                  cy="50%"
                  outerRadius={100}
                  fill="#8884d8"
                  dataKey="value"
                  label
                >
                  {memoryData.map((entry, index) => (
                    <Cell
                      key={`cell-${index}`}
                      fill={COLORS[index % COLORS.length]}
                    />
                  ))}
                </Pie>
                <Tooltip formatter={(value) => `${value.toFixed(2)} GB`} />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
            <ResponsiveContainer width="45%" height={400}>
              <PieChart>
                <Pie
                  data={swapData}
                  cx="50%"
                  cy="50%"
                  outerRadius={100}
                  fill="#8884d8"
                  dataKey="value"
                  label
                >
                  {swapData.map((entry, index) => (
                    <Cell
                      key={`cell-${index}`}
                      fill={COLORS[index % COLORS.length]}
                    />
                  ))}
                </Pie>
                <Tooltip formatter={(value) => `${value.toFixed(2)} GB`} />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          </>
        )}
      </div>
    </div>
  );
};

export default SysInfo;
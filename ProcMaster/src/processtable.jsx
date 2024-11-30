import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./processtable.css"; // Optional for custom styling

const ProcessTable = () => {
  const [processes, setProcesses] = useState([]);
  const [loading, setLoading] = useState(true);
  const [sortConfig, setSortConfig] = useState({ key: "pid", direction: "asc" });
  const [originalProcesses, setOriginalProcesses] = useState([]); // Track original order

  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        const result = await invoke("get_processess");
        setProcesses(result);
        setOriginalProcesses(result); // Store original data
        setLoading(false);
      } catch (error) {
        console.error("Error fetching processes:", error);
        setLoading(false);
      }
    };

    fetchProcesses();
  }, []);

  const sortProcesses = (key, direction) => {
    const sortedProcesses = [...processes].sort((a, b) => {
      if (a[key] < b[key]) return direction === "asc" ? -1 : 1;
      if (a[key] > b[key]) return direction === "asc" ? 1 : -1;
      return 0;
    });
    setProcesses(sortedProcesses);
  };

  const handleSort = (key) => {
    let newDirection = "asc";
    if (key === sortConfig.key) {
      if (sortConfig.direction === "asc") {
        newDirection = "desc";
      } else if (sortConfig.direction === "desc") {
        setProcesses(originalProcesses); // Reset to original state on third click
        setSortConfig({ key: "", direction: "" }); // Reset sort config
        return;
      }
    }
    setSortConfig({ key, direction: newDirection });
    sortProcesses(key, newDirection);
  };

  if (loading) {
    return <div>Loading processes...</div>;
  }

  return (
    <div className="process-table-container">
      <table className="process-table">
        <thead>
          <tr>
            {["pid", "user", "command", "v_memory", "rss_memory", "shared_memory", "memory_usage", "cpu_usage", "time", "priority", "nice", "ppid", "state", "threads"].map((header) => (
              <th key={header} onClick={() => handleSort(header)}>
                {header.charAt(0).toUpperCase() + header.slice(1).replace(/_/g, " ")}
                {sortConfig.key === header && (sortConfig.direction === "asc" ? " ↑" : " ↓")}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {processes.map((process, index) => (
            <tr key={index}>
              <td>{process.pid}</td>
              <td>{process.user}</td>
              <td>{process.command}</td>
              <td>{process.v_memory.toFixed(2)}</td>
              <td>{process.rss_memory.toFixed(2)}</td>
              <td>{process.shared_memory.toFixed(2)}</td>
              <td>{process.memory_uasge.toFixed(2)}</td>
              <td>{process.cpu_usage.toFixed(2)}</td>
              <td>{process.time}</td>
              <td>{process.priority}</td>
              <td>{process.nice}</td>
              <td>{process.ppid}</td>
              <td>{process.state}</td>
              <td>{process.threads}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default ProcessTable;

import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./processtable.css"; // Optional for custom styling

const ProcessTable = () => {
  const [processes, setProcesses] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        const result = await invoke("get_processess");
        setProcesses(result);
        setLoading(false);
      } catch (error) {
        console.error("Error fetching processes:", error);
        setLoading(false);
      }
    };

    fetchProcesses();
  }, []);

  if (loading) {
    return <div>Loading processes...</div>;
  }

  return (
    <div className="process-table-container">
      <table className="process-table">
        <thead>
          <tr>
            <th>PID</th>
            <th>User</th>
            <th>Command</th>
            <th>Virtual Memory (MB)</th>
            <th>RSS Memory (MB)</th>
            <th>Shared Memory (MB)</th>
            <th>Memory Usage (%)</th>
            <th>CPU Usage (%)</th>
            <th>Time</th>
            <th>Priority</th>
            <th>Nice</th>
            <th>Parent PID</th>
            <th>State</th>
            <th>Threads</th>
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

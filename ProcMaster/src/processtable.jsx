import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./processtable.css"; // Optional for custom styling

const ProcessTable = () => {
  const [processes, setProcesses] = useState([]);
  const [loading, setLoading] = useState(true);
  const [sortConfig, setSortConfig] = useState({ key: "pid", direction: "asc" });
  const [originalProcesses, setOriginalProcesses] = useState([]); // Track original order
  const [contextMenu, setContextMenu] = useState(null); // State for context menu visibility
  const [selectedProcess, setSelectedProcess] = useState(null); // Selected process for actions
  const [showPriorityDialog, setShowPriorityDialog] = useState(false);
  const [selectedPriority, setSelectedPriority] = useState(0);
  const [errorMessage, setErrorMessage] = useState('');
  const [password, setPassword] = useState('');


  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        const result = await invoke("get_processess");
        setOriginalProcesses(result); // Store original data
        let updatedProcesses = [...result];
        
        // Reapply current sort configuration
        if (sortConfig.key) {
          updatedProcesses = sortProcesses(updatedProcesses, sortConfig.key, sortConfig.direction);
        }
        
        setProcesses(updatedProcesses); // Update processes with sorting applied
        setLoading(false);
      } catch (error) {
        console.error("Error fetching processes:", error);
        setLoading(false);
      }
    };

    fetchProcesses();

    const interval = setInterval(() => {
      fetchProcesses();
    }, 1000);

    // Clean up the interval on component unmount
    return () => clearInterval(interval);

  }, [sortConfig]); // Re-run when sortConfig changes

  const sortProcesses = (data, key, direction) => {
    return [...data].sort((a, b) => {
      if (a[key] < b[key]) return direction === "asc" ? -1 : 1;
      if (a[key] > b[key]) return direction === "asc" ? 1 : -1;
      return 0;
    });
  };

  const handleAction = async (pid, action) => {
    try {
      switch (action) {
        case "kill":
          await invoke("kill_process", { pid });
          break;
        case "terminate":
          await invoke("terminate_process", { pid });
          break;
        case "suspend":
          await invoke("suspend_process", { pid });
          break;
        case "resume":
          await invoke("resume_process", { pid });
          break;
        case "priority":
          setSelectedPriority(selectedProcess.nice || 0); // Initialize with current nice value
          setErrorMessage('');
          setShowPriorityDialog(true);
          break;
        default:
          break;
      }

      if (action === "kill" || action === "terminate") 
      {
        // Remove the killed process from the processes state
        setProcesses((prevProcesses) => prevProcesses.filter((process) => process.pid !== pid));
      }
    } catch (error) {
      console.error("Error performing action on process:", error);
    }
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
    const sortedProcesses = sortProcesses(processes, key, newDirection);
    setSortConfig({ key, direction: newDirection });
    setProcesses(sortedProcesses);
  };

  const handleRightClick = (e, process) => {
    e.preventDefault(); // Prevent default right-click menu
    setSelectedProcess(process); // Set the selected process
    const { clientX: mouseX, clientY: mouseY } = e;
    setContextMenu({ x: mouseX, y: mouseY }); // Position context menu
  };

  const handleCloseMenu = () => {
    setContextMenu(null); // Close the context menu
  };

  useEffect(() => {
    const handleClickOutside = (e) => {
      if (contextMenu && !e.target.closest(".custom-context-menu")) {
        handleCloseMenu();
      }
    };

    document.addEventListener("click", handleClickOutside);

    // Cleanup the event listener on component unmount
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  }, [contextMenu]);

  const handleChangePriority = async () => {
    try {
      const result = await invoke("change_priority", {
        pid: selectedProcess.pid,
        priority: selectedPriority,
        password,
      });
      if (result) {
        setShowPriorityDialog(false);
      } else {
        setErrorMessage("Failed to change priority. Incorrect password or insufficient permissions.");
      }
    } catch (error) {
      console.error("Error changing priority:", error);
      setErrorMessage("An error occurred while changing priority.");
    }
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
            <tr
              key={index}
              onContextMenu={(e) => handleRightClick(e, process)} // Add right-click handler to each row
            >
              <td>{process.pid}</td>
              <td>{process.user}</td>
              <td>{process.command}</td>
              <td>{process.v_memory?.toFixed(2)}</td>
              <td>{process.rss_memory?.toFixed(2)}</td>
              <td>{process.shared_memory?.toFixed(2)}</td>
              <td>{process.memory_uasge?.toFixed(2)}</td>
              <td>{process.cpu_usage?.toFixed(2)}</td>
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

      {contextMenu && selectedProcess && (
        <div
          className="custom-context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
          onClick={handleCloseMenu} // Close the menu on click outside
        >
          <ul>
            <li onClick={() => handleAction(selectedProcess.pid, "kill")}>Kill</li>
            <li onClick={() => handleAction(selectedProcess.pid, "terminate")}>Terminate</li>
            <li onClick={() => handleAction(selectedProcess.pid, "suspend")}>Suspend</li>
            <li onClick={() => handleAction(selectedProcess.pid, "resume")}>Resume</li>
            <li onClick={() => handleAction(selectedProcess.pid, "priority")}>Change Priority</li>
          </ul>
        </div>
      )}

      {showPriorityDialog && selectedProcess && (
        <div className="priority-dialog-overlay">
          <div className="priority-dialog-content">
            <h3>Change Priority for PID {selectedProcess.pid}</h3>
            <div className="input-group">
              <label>
                Priority (-20 to 19): <strong>{selectedPriority}</strong>
              </label>
              <input
                type="range"
                min="-20"
                max="19"
                step="1"
                value={selectedPriority}
                onChange={(e) => setSelectedPriority(Number(e.target.value))}
              />
            </div>
            <div className="input-group">
              <label>Password:</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            {errorMessage && <p className="error-message">{errorMessage}</p>}
            <div className="dialog-buttons">
              <button onClick={handleChangePriority}>Change Priority</button>
              <button onClick={() => setShowPriorityDialog(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ProcessTable;

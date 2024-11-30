import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import TabContext from '@mui/lab/TabContext';
import TabList from '@mui/lab/TabList';
import TabPanel from '@mui/lab/TabPanel';
import TableRowsIcon from '@mui/icons-material/TableRows';
import MemoryIcon from '@mui/icons-material/Memory';
import SpeedIcon from '@mui/icons-material/Speed';
import SysInfo from "./sysinfo"; 
import ProcessTable from "./processtable";
function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function fetchProcesses() {
    try {
      const processes = await invoke('get_processes_info');
      console.log("Processes: ", processes);
      // Update your UI with the fetched process data
    } catch (error) {
      console.error("Failed to fetch processes:", error);
    }
  }
  

  const [value, setValue] = useState('1');

  const handleValueChange = (event, newValue) => {
    setValue(newValue);
  };


  return (
    <main className="container">
      <TabContext value={value}>
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <TabList onChange={handleValueChange} centered>
            <Tab label="Processes" value="1" icon={<TableRowsIcon/>} iconPosition="start"  sx={{ flexGrow: 1 }} />
            <Tab label="Resources" value="2" icon={<SpeedIcon/>} iconPosition="start"  sx={{ flexGrow: 1 }} />
          </TabList>
        </Box>
          <TabPanel  value="1" style={{padding: 0, margin: 0 }}>
            
              <ProcessTable/>
          </TabPanel>
          <TabPanel value="2" style={{padding: 0, margin: 0}}>
              <SysInfo/>
          </TabPanel>
      </TabContext>
    </main>
  );
}

export default App;

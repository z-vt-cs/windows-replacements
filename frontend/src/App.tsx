import { useState, useEffect } from 'react'
import './App.css'
import {invoke} from '@tauri-apps/api/core'

type StartIconProps = {
  size?: number
  color?: string
}

function TasbarIcons() {
  const [windows, setWindows] = useState<string[]>([])

  useEffect(() => {
    invoke<string[]>('get_taskbar_windows').then(setWindows)
  }, [])

  return (
    <div>
      {windows.map((window, index) => (
        <div key={index} className="taskbar-icon">
          {window}
        </div>
      ))}
    </div>
  )
}

function StartIcon({ size = 24, color = '#fff' }: StartIconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={color}
      style={{ position: 'absolute', left: 10, top: '50%', transform: 'translateY(-50%)' }}
    >
      <rect x="3" y="3" width="8" height="8" />
      <rect x="13" y="3" width="8" height="8" />
      <rect x="3" y="13" width="8" height="8" />
      <rect x="13" y="13" width="8" height="8" />
    </svg>
  )
}

type OpenProcessIconProps = {
  size?: number
  color?: string
}


function App() {
  const [count, setCount] = useState(0)

  return (
    <div className = "taskbar">
      <div className="windows-icon" 
        style={{ position: "absolute", 
        left : 0, 
        top: 0, 
        width: "100%", 
        height: "100%" 
        }}>

          <StartIcon size={24} color="#fff" />
          <TasbarIcons />

      </div>
    </div>
  )
}

export default App

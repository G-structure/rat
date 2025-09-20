import { Component } from "solid-js";

const TestApp: Component = () => {
  return (
    <div style={{ padding: "20px", "font-family": "sans-serif" }}>
      <h1>RAT Mobile IDE - Test</h1>
      <p>If you can see this, SolidJS is working!</p>
      <button 
        style={{
          padding: "10px 20px",
          background: "#3b82f6",
          color: "white",
          border: "none",
          "border-radius": "8px",
          cursor: "pointer"
        }}
        onClick={() => alert("App is working!")}
      >
        Click Me
      </button>
    </div>
  );
};

export default TestApp;
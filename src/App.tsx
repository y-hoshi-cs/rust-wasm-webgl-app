import { useEffect, useState, useRef, useCallback, useReducer } from 'react'
import './App.css'
import init, { init_gl, Screen } from './../wasm/pkg'

interface CanvasState {
  id: string;
  width: number;
  height: number;
  diskNum: number;
  diskSize: number;
}

const canvasState: CanvasState = {
  id: "canvas",
  width: 400,
  height: 400,
  diskNum: 100,
  diskSize: 16,
};

const ACTION_TYPES = [
  "WINDOW_CHANGE",
  "DISK_SIZE_CHANGE",
  "DISK_NUM_CHANGE",
  "COLLISION_CHANGE",
] as const;

type ADT<Ident extends string, T> = {
  [K in keyof T]: Record<Ident, K> & T[K];
}[keyof T];

type DispatchActions<Ks extends keyof any, T extends Record<Ks, any>> = ADT<"type", T>;

type ActionTypes = typeof ACTION_TYPES[number];

type Actions = DispatchActions<
  ActionTypes,
  {
    WINDOW_CHANGE: { payload: string };
    DISK_SIZE_CHANGE: { payload: string };
    DISK_NUM_CHANGE: { payload: string };
    COLLISION_CHANGE: { payload: boolean };
  }
>;

const reducer = (state: CanvasState, action: Actions) => {
  switch (action.type) {
    case ACTION_TYPES[0]:
      return { ...state, width: parseInt(action.payload, 10), height: parseInt(action.payload, 10) };
    case ACTION_TYPES[1]:
      return { ...state, diskSize: parseInt(action.payload, 10) };
    case ACTION_TYPES[2]:
      return { ...state, diskNum: parseInt(action.payload, 10) };
    case ACTION_TYPES[3]:
      return { ...state, collision: action.payload };
    default:
      return state;
  }
}

function App() {
  const t = useRef<number>(0);
  const glInstance = useRef<Screen | null>(null);
  const req = useRef<number | null>(null);
  const [animate, setAnimate] = useState<boolean>(false);
  const [state, dispatch] = useReducer(reducer, canvasState)

  const doFrame = () => {
    if (!glInstance.current) return;
    t.current += 1;
    glInstance.current.do_frame();
    req.current = requestAnimationFrame(doFrame);
  }

  const initGl = () => {
    glInstance.current = init_gl({
      canvas_id: state.id,
      disk_num: state.diskNum,
      width: state.width,
      height: state.height,
      disk_size: state.diskSize,
    });
    glInstance.current.do_frame();
  }

  useEffect(() => {
    const initialize = async () => {
      await init();
      initGl(); 
    }
    initialize();
  }, [])

  useEffect(() => {
    if (!glInstance.current) return;
    initGl();
  }, [glInstance, state])

  const toggleAnimationState = useCallback(() => {
    if (animate === true && req.current !== null) {
      setAnimate(false);
      cancelAnimationFrame(req.current);
      req.current = null;
    } else {
      setAnimate(true);
      doFrame();
    }
  }, [animate]);
  
  return (
    <div className="App">
      <div>
        <button onClick={toggleAnimationState}>
          {animate === true ? 'stop' : 'start'}
        </button>
        <select value={state.diskSize} onChange={(e) => dispatch({ type: "DISK_SIZE_CHANGE", payload: e.target.value })}>
          <option value="4">4</option>
          <option value="8">8</option>
          <option value="16">16</option>
          <option value="32">32</option>
          <option value="64">64</option>
        </select>
        <select value={state.diskNum} onChange={(e) => dispatch({ type: "DISK_NUM_CHANGE", payload: e.target.value })}>
          <option value="10">10</option>
          <option value="100">100</option>
          <option value="1000">1000</option>
          <option value="10000">10000</option>
          <option value="50000">50000</option>
          <option value="100000">100000</option>
        </select>
        <select value={state.height} onChange={(e) => dispatch({ type: "WINDOW_CHANGE", payload: e.target.value })}>
          <option value="200">200</option>
          <option value="400">400</option>
          <option value="600">600</option>
          <option value="800">800</option>
          <option value="1000">1000</option>
        </select>
      </div>
      <canvas id={state.id} width={state.width} height={state.height} />
    </div>
  )
}

export default App

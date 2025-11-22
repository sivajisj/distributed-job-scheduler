 // frontend/components/WebSocketProvider.tsx
'use client';

import React, { 
    createContext, 
    useContext, 
    useEffect, 
    useState, 
    useCallback 
} from 'react';

// Define the shape of the job data (simplified)
export interface Job {
    id: string;
    job_type: string;
    status: string;
    created_at: string;
    started_at: string | null;
    finished_at: string | null;
    result: any | null;
}

// Context State
interface WebSocketContextType {
    jobs: Job[];
    loading: boolean;
    sendMessage: (message: string) => void;
    connectionStatus: 'connecting' | 'open' | 'closed';
}

const WebSocketContext = createContext<WebSocketContextType | undefined>(undefined);

// Define the WS server URL (use environment variables in a real app)
const REST_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/jobs';
const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8000/ws';

export const WebSocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    // Stores the list of all jobs, indexed by ID for fast updates
    const [jobMap, setJobMap] = useState<Map<string, Job>>(new Map());
    const [ws, setWs] = useState<WebSocket | null>(null);
    const [loading, setLoading] = useState(true);
    const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'open' | 'closed'>('connecting');

    // 1. Fetch initial job list from REST API
    const fetchInitialJobs = useCallback(async () => {
        try {
            const response = await fetch(REST_URL);
            const initialJobs: Job[] = await response.json();
            const newMap = new Map<string, Job>();
            initialJobs.forEach(job => newMap.set(job.id, job));
            setJobMap(newMap);
        } catch (error) {
            console.error('Failed to fetch initial jobs:', error);
        } finally {
            setLoading(false);
        }
    }, []);

    // 2. WebSocket Connection Logic
    useEffect(() => {
        fetchInitialJobs();

        const socket = new WebSocket(WS_URL);
        setWs(socket);
        setConnectionStatus('connecting');

        socket.onopen = () => {
            console.log('WebSocket connected');
            setConnectionStatus('open');
        };

        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                // Check for the expected WsMessage structure
                if (message.type === 'JobStatusUpdate') {
                    const newJob: Job = message.data;
                    // Update the job map immutably
                    setJobMap(prevMap => {
                        const newMap = new Map(prevMap);
                        newMap.set(newJob.id, newJob);
                        return newMap;
                    });
                }
            } catch (e) {
                console.error('Error parsing WS message:', e);
            }
        };

        socket.onclose = () => {
            console.log('WebSocket closed. Attempting reconnect...');
            setConnectionStatus('closed');
            // Simple reconnect logic for MVP
            setTimeout(() => {
                 setWs(null); // Force useEffect cleanup and restart
            }, 3000); 
        };

        socket.onerror = (error) => {
            console.error('WebSocket error:', error);
            setConnectionStatus('closed');
        };

        // Cleanup function
        return () => {
            socket.close();
        };
    }, [fetchInitialJobs]);

    // Simple API to send messages (optional for a read-only dashboard)
    const sendMessage = useCallback((message: string) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(message);
        }
    }, [ws]);

    // Convert map back to array for display, sorted by creation time
    const jobsArray = Array.from(jobMap.values()).sort((a, b) => 
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );

    return (
        <WebSocketContext.Provider value={{ jobs: jobsArray, loading, sendMessage, connectionStatus }}>
            {children}
        </WebSocketContext.Provider>
    );
};

export const useWebSocket = () => {
    const context = useContext(WebSocketContext);
    if (!context) {
        throw new Error('useWebSocket must be used within a WebSocketProvider');
    }
    return context;
};
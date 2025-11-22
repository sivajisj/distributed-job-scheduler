// frontend/app/page.tsx
'use client';

import { useState } from 'react';
import { JobTable } from './components/JobTable';
import { useWebSocket } from './components/WebSocketProvider';

const REST_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/jobs';
export default function Dashboard() {
    const { connectionStatus } = useWebSocket();
    const [jobType, setJobType] = useState('process_data');
    const [payload, setPayload] = useState('{"user_id": 101, "priority": "high"}');
    const [statusMessage, setStatusMessage] = useState('');

    const createJob = async () => {
        try {
            const jsonPayload = JSON.parse(payload);
            const response = await fetch(REST_URL, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ job_type: jobType, payload: jsonPayload }),
            });

            if (response.ok) {
                const job = await response.json();
                setStatusMessage(`Job created successfully: ${job.id}`);
            } else {
                setStatusMessage(`Failed to create job: ${response.statusText}`);
            }
        } catch (e) {
            setStatusMessage(`Invalid JSON payload or API error: ${e instanceof Error ? e.message : String(e)}`);
        }
    };
    
    const wsStatusClass = connectionStatus === 'open' ? 'bg-green-500' : 'bg-red-500';

    return (
        <div className="min-h-screen bg-gray-100 p-8">
            <header className="flex justify-between items-center mb-6">
                <h1 className="text-3xl font-bold text-gray-800">🦀 Rust Job Scheduler Dashboard</h1>
                <div className={`text-sm font-medium text-white px-3 py-1 rounded-full ${wsStatusClass}`}>
                    WS: {connectionStatus.toUpperCase()}
                </div>
            </header>

            {/* Job Creation Form */}
            <div className="bg-white shadow-xl rounded-lg p-6 mb-8">
                <h2 className="text-xl font-semibold mb-4">Submit New Job</h2>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <input 
                        type="text" 
                        placeholder="Job Type (e.g., process_data or fail_job)"
                        value={jobType} 
                        onChange={(e) => setJobType(e.target.value)}
                        className="p-3 border rounded-md focus:ring-blue-500 focus:border-blue-500"
                    />
                    <textarea 
                        placeholder="JSON Payload (e.g., {'data': 'value'})"
                        value={payload} 
                        onChange={(e) => setPayload(e.target.value)}
                        rows={3}
                        className="p-3 border rounded-md md:col-span-2 focus:ring-blue-500 focus:border-blue-500 font-mono"
                    />
                    <button 
                        onClick={createJob}
                        className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-4 rounded-md transition duration-150 ease-in-out md:col-span-3"
                    >
                        Queue Job
                    </button>
                </div>
                {statusMessage && <p className="mt-4 text-sm text-gray-600">{statusMessage}</p>}
            </div>

            {/* Job Monitoring Table */}
            <h2 className="text-2xl font-semibold mb-4 text-gray-800">Live Job Status</h2>
            <JobTable />
        </div>
    );
}
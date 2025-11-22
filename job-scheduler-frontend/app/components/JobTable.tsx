// frontend/components/JobTable.tsx
'use client';

import React from 'react';
import { Job, useWebSocket } from './WebSocketProvider';

const statusClasses: Record<string, string> = {
    QUEUED: 'bg-gray-200 text-gray-800',
    RUNNING: 'bg-blue-200 text-blue-800 animate-pulse',
    COMPLETED: 'bg-green-200 text-green-800',
    FAILED: 'bg-red-200 text-red-800',
};

const JobRow: React.FC<{ job: Job }> = ({ job }) => {
    const statusText = job.status.toUpperCase();
    const statusClass = statusClasses[statusText] || statusClasses.QUEUED;

    const formatDate = (dateString: string | null) => 
        dateString ? new Date(dateString).toLocaleTimeString() : 'N/A';

    return (
        <tr className="border-b hover:bg-gray-50 text-sm">
            <td className="p-3 font-mono text-xs text-gray-600">{job.id.substring(0, 8)}...</td>
            <td className="p-3 font-semibold">{job.job_type}</td>
            <td className="p-3">
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusClass}`}>
                    {statusText}
                </span>
            </td>
            <td className="p-3">{formatDate(job.created_at)}</td>
            <td className="p-3">{formatDate(job.finished_at)}</td>
            <td className="p-3 max-w-xs overflow-hidden text-ellipsis whitespace-nowrap">
                {job.result ? JSON.stringify(job.result) : 'N/A'}
            </td>
        </tr>
    );
};

export const JobTable: React.FC = () => {
    const { jobs, loading } = useWebSocket();

    if (loading) {
        return <div className="p-4 text-center text-xl">Loading jobs...</div>;
    }

    return (
        <div className="bg-white shadow-lg rounded-lg overflow-hidden">
            <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-100">
                    <tr>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">ID</th>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Type</th>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Created</th>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Finished</th>
                        <th className="p-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Result</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-200">
                    {jobs.length > 0 ? (
                        jobs.map(job => <JobRow key={job.id} job={job} />)
                    ) : (
                        <tr><td colSpan={6} className="p-4 text-center text-gray-500">No jobs found.</td></tr>
                    )}
                </tbody>
            </table>
        </div>
    );
};
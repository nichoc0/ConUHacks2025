import React, { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts';
import { AlertTriangle, Activity, Database, Shield } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';

const NetworkDashboard = () => {
  const [networkData, setNetworkData] = useState({
    tcpEvents: [],
    udpEvents: [],
    arpEvents: [],
    dnsEvents: [],
    suspiciousEvents: []
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const responses = await Promise.all([
          fetch('http://localhost:3000/api/events/tcp'),
          fetch('http://localhost:3000/api/events/udp'),
          fetch('http://localhost:3000/api/events/arp'),
          fetch('http://localhost:3000/api/events/dns'),
          fetch('http://localhost:3000/api/events/suspicious')
        ]);

        const [tcp, udp, arp, dns, suspicious] = await Promise.all(
          responses.map(r => r.json())
        );

        setNetworkData({
          tcpEvents: tcp,
          udpEvents: udp,
          arpEvents: arp,
          dnsEvents: dns,
          suspiciousEvents: suspicious
        });
        setLoading(false);
      } catch (err) {
        setError(err.message);
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 60000); // Refresh every minute
    return () => clearInterval(interval);
  }, []);

  if (loading) return <div className="flex items-center justify-center h-screen">Loading...</div>;
  if (error) return <div className="text-red-500">Error: {error}</div>;

  const trafficData = [
    { name: 'TCP', value: networkData.tcpEvents.length },
    { name: 'UDP', value: networkData.udpEvents.length },
    { name: 'ARP', value: networkData.arpEvents.length },
    { name: 'DNS', value: networkData.dnsEvents.length }
  ];

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-4">Network Monitoring Dashboard</h1>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            title="Total Events"
            value={Object.values(networkData).flat().length}
            icon={<Activity className="h-6 w-6" />}
          />
          <StatCard
            title="Suspicious Activities"
            value={networkData.suspiciousEvents.length}
            icon={<AlertTriangle className="h-6 w-6" />}
            alert={true}
          />
          <StatCard
            title="DNS Queries"
            value={networkData.dnsEvents.length}
            icon={<Database className="h-6 w-6" />}
          />
          <StatCard
            title="Security Events"
            value={networkData.suspiciousEvents.filter(e => 
              e.activity_type.includes('Port Scanning') || 
              e.activity_type.includes('ARP Spoofing')
            ).length}
            icon={<Shield className="h-6 w-6" />}
          />
        </div>
      </div>

      <div className="mb-8">
        <h2 className="text-2xl font-bold mb-4">Traffic Distribution</h2>
        <div className="bg-white p-4 rounded-lg shadow">
          <LineChart width={800} height={300} data={trafficData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="value" stroke="#8884d8" />
          </LineChart>
        </div>
      </div>

      <div className="mb-8">
        <h2 className="text-2xl font-bold mb-4">Suspicious Activities</h2>
        <div className="space-y-4">
          {networkData.suspiciousEvents.map((event, index) => (
            <Alert key={index} variant={event.activity_type.includes('Port Scanning') ? 'destructive' : 'default'}>
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>{event.activity_type}</AlertTitle>
              <AlertDescription>
                Source: {event.source}<br />
                Details: {event.details}<br />
                Time: {new Date(event.timestamp * 1000).toLocaleString()}
              </AlertDescription>
            </Alert>
          ))}
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <EventList
          title="Recent TCP Events"
          events={networkData.tcpEvents.slice(-5)}
        />
        <EventList
          title="Recent UDP Events"
          events={networkData.udpEvents.slice(-5)}
        />
      </div>
    </div>
  );
};

const StatCard = ({ title, value, icon, alert }) => (
  <div className={`p-6 rounded-lg shadow ${alert && value > 0 ? 'bg-red-50' : 'bg-white'}`}>
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm text-gray-600">{title}</p>
        <p className="text-2xl font-bold">{value}</p>
      </div>
      <div className={`${alert && value > 0 ? 'text-red-500' : 'text-blue-500'}`}>
        {icon}
      </div>
    </div>
  </div>
);

const EventList = ({ title, events }) => (
  <div className="bg-white p-4 rounded-lg shadow">
    <h3 className="text-xl font-bold mb-4">{title}</h3>
    <div className="space-y-2">
      {events.map((event, index) => (
        <div key={index} className="p-2 bg-gray-50 rounded">
          <p className="text-sm">
            <span className="font-medium">Source:</span> {event.source}
          </p>
          <p className="text-sm">
            <span className="font-medium">Destination:</span> {event.destination}
          </p>
          <p className="text-sm">
            <span className="font-medium">Size:</span> {event.payload_size} bytes
          </p>
        </div>
      ))}
    </div>
  </div>
);

export default NetworkDashboard;
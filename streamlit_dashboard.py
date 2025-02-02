import streamlit as st
import pandas as pd
from datetime import datetime, timedelta
import time
import plotly.graph_objects as go
from motor.motor_asyncio import AsyncIOMotorClient
import asyncio
from threading import Thread

def format_timestamp(ts):
    return datetime.fromtimestamp(ts).strftime('%Y-%m-%d %H:%M:%S')

st.title('🔒 Network Security Monitor')

# Initialize session state for data and loading states
if 'network_events' not in st.session_state:
    st.session_state.network_events = pd.DataFrame(columns=['timestamp', 'protocol', 'source', 'destination', 'payload_size'])
if 'suspicious_events' not in st.session_state:
    st.session_state.suspicious_events = pd.DataFrame(columns=['timestamp', 'activity_type', 'source', 'details'])
if 'is_loading' not in st.session_state:
    st.session_state.is_loading = False

@st.cache_resource(show_spinner=False)
def init_connection():
    return AsyncIOMotorClient('mongodb://localhost:27017/')

try:
    client = init_connection()
    db = client.network_monitor
except Exception as e:
    st.error(f"Failed to connect to MongoDB: {str(e)}")
    st.stop()

async def fetch_events(start_time, end_time):
    events = []
    collections = ['tcp_events', 'udp_events', 'arp_events', 'dns_events']
    for collection_name in collections:
        collection = db[collection_name]
        cursor = collection.find({
            'timestamp': {'$gte': start_time, '$lte': end_time}
        })
        async for doc in cursor:
            events.append(doc)
    return events

async def fetch_suspicious_events(start_time, end_time):
    events = []
    cursor = db['sus_events'].find({
        'timestamp': {'$gte': start_time, '$lte': end_time}
    })
    async for doc in cursor:
        events.append(doc)
    return events

# Event Timeline Section
st.header('Event Timeline (Last 10 Minutes)')

# Create time bins and empty figure
now = datetime.now()
ten_mins_ago = now - timedelta(minutes=10)
bins = pd.date_range(start=ten_mins_ago, end=now, freq='30S')
fig = go.Figure()
fig.update_layout(
    barmode='stack',
    xaxis_title='Time',
    yaxis_title='Event Count',
    height=400,
    xaxis=dict(
        range=[ten_mins_ago, now],
        rangeslider=dict(visible=True),
        type='date'
    ),
    yaxis=dict(
        rangemode='nonnegative'
    ),
    showlegend=True
)

# Display empty figure first
timeline_container = st.container()
with timeline_container:
    st.plotly_chart(fig, use_container_width=True)

# Network Events Section
st.header('Network Events')
events_container = st.container()
with events_container:
    st.dataframe(
        st.session_state.network_events,
        use_container_width=True,
        hide_index=True
    )

# Suspicious Activities Section
st.header('Suspicious Activities')
suspicious_container = st.container()
with suspicious_container:
    st.dataframe(
        st.session_state.suspicious_events,
        use_container_width=True,
        hide_index=True
    )

async def load_data():
    if st.session_state.is_loading:
        return

    try:
        st.session_state.is_loading = True
        
        # Run both queries concurrently
        normal_events, suspicious_events = await asyncio.gather(
            fetch_events(ten_mins_ago.timestamp(), now.timestamp()),
            fetch_suspicious_events(ten_mins_ago.timestamp(), now.timestamp())
        )
        
        # Update the session state with new data
        if normal_events:
            df = pd.DataFrame(normal_events)
            df['timestamp'] = df['timestamp'].apply(format_timestamp)
            st.session_state.network_events = df
            
        if suspicious_events:
            df = pd.DataFrame(suspicious_events)
            df['timestamp'] = df['timestamp'].apply(format_timestamp)
            st.session_state.suspicious_events = df
            
        return normal_events, suspicious_events
    except Exception as e:
        st.error(f"Error loading data: {str(e)}")
        return None, None
    finally:
        st.session_state.is_loading = False

# Show loading indicator
if st.session_state.is_loading:
    st.spinner('Loading data...')

# Refresh button
if st.button('Refresh Data'):
    asyncio.run(load_data())

# Auto-refresh every 5 seconds if not loading
if not st.session_state.is_loading and time.time() - st.session_state.get('last_refresh', 0) > 5:
    st.session_state.last_refresh = time.time()
    asyncio.run(load_data())

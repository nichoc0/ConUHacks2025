import streamlit as st
import pandas as pd
from datetime import datetime, timedelta
import time
import plotly.graph_objects as go
from motor.motor_asyncio import AsyncIOMotorClient
import asyncio
import nest_asyncio

try:
    # Enable nested event loops
    nest_asyncio.apply()

    def format_timestamp(ts):
        return datetime.fromtimestamp(ts).strftime('%Y-%m-%d %H:%M:%S')

    st.title('🦀 Network Security Crab-board')

    # Initialize session state
    if 'network_events' not in st.session_state:
        st.session_state.network_events = pd.DataFrame(columns=['timestamp', 'protocol', 'source', 'destination', 'payload_size'])
    if 'suspicious_events' not in st.session_state:
        st.session_state.suspicious_events = pd.DataFrame(columns=['timestamp', 'activity_type', 'source', 'details'])
    if 'is_loading' not in st.session_state:
        st.session_state.is_loading = False
    if 'loop' not in st.session_state:
        st.session_state.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(st.session_state.loop)

    @st.cache_resource(show_spinner=False)
    def init_db():
        try:
            client = AsyncIOMotorClient('mongodb://localhost:27017/', serverSelectionTimeoutMS=1000)
            return client.network_monitor
        except Exception:
            return None

    db = init_db()

    async def fetch_events(start_time, end_time):
        if not db:
            return []
        try:
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
        except Exception:
            return []

    async def fetch_suspicious_events(start_time, end_time):
        if not db:
            return []
        try:
            events = []
            cursor = db['sus_events'].find({
                'timestamp': {'$gte': start_time, '$lte': end_time}
            })
            async for doc in cursor:
                events.append(doc)
            return events
        except Exception:
            return []

    # Event Timeline Section
    st.header('Event Timeline (Last 10 Minutes)')

    # Create timeline container first
    timeline_container = st.container()

    def update_timeline(normal_events=None, suspicious_events=None):
        now = datetime.now()
        ten_mins_ago = now - timedelta(minutes=10)
        bins = pd.date_range(start=ten_mins_ago, end=now, freq='30S')
        
        fig = go.Figure()
        
        if normal_events:
            normal_df = pd.DataFrame(normal_events)
            normal_df['datetime'] = pd.to_datetime(normal_df['timestamp'].apply(lambda x: datetime.fromtimestamp(x)))
            normal_counts = pd.cut(normal_df['datetime'], bins=bins).value_counts().sort_index()
            fig.add_trace(go.Bar(
                x=bins[:-1],
                y=normal_counts.values,
                name='Normal Events',
                marker_color='green'
            ))
        
        if suspicious_events:
            sus_df = pd.DataFrame(suspicious_events)
            sus_df['datetime'] = pd.to_datetime(sus_df['timestamp'].apply(lambda x: datetime.fromtimestamp(x)))
            sus_counts = pd.cut(sus_df['datetime'], bins=bins).value_counts().sort_index()
            fig.add_trace(go.Bar(
                x=bins[:-1],
                y=sus_counts.values,
                name='Suspicious Events',
                marker_color='red'
            ))
        
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
        
        with timeline_container:
            st.plotly_chart(fig, use_container_width=True)

    # Initialize empty timeline
    update_timeline()

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

        st.session_state.is_loading = True
        try:
            if not db:
                st.warning("MongoDB is not running. Start the backend to see live data.", icon="⚠️")
                return None, None
                
            now = datetime.now()
            ten_mins_ago = now - timedelta(minutes=10)
            
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
                
            # Update the timeline with the new data
            update_timeline(normal_events, suspicious_events)
                    
            return normal_events, suspicious_events
        except Exception:
            # Silently handle errors when backend is not running
            return None, None
        finally:
            st.session_state.is_loading = False

    # Show loading indicator
    if st.session_state.is_loading:
        st.spinner('Loading data...')

    # Refresh button
    if st.button('Refresh Data'):
        st.session_state.loop.run_until_complete(load_data())

    # Auto-refresh every 5 seconds if not loading
    if not st.session_state.is_loading and time.time() - st.session_state.get('last_refresh', 0) > 5:
        st.session_state.last_refresh = time.time()
        st.session_state.loop.run_until_complete(load_data())
except:
    pass
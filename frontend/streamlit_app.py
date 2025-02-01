import streamlit as st
import threading
import queue
import time
from datetime import datetime

# Import our Rust module
import sniff

# Thread-safe queue for events
event_queue = queue.Queue()

def rust_event_callback(event):
    """Callback function that receives events from Rust"""
    event_queue.put({
        "time": datetime.now().strftime("%H:%M:%S"),
        "protocol": event.protocol,
        "source": event.source,
        "destination": event.destination,
        "size": event.payload_size
    })

def start_monitoring():
    """Start the Rust network monitor in background"""
    sniff.start_sniffing_py(None, rust_event_callback)

# Initialize session state
if "events" not in st.session_state:
    st.session_state.events = []
    
if "monitor_started" not in st.session_state:
    st.session_state.monitor_started = False

# Main dashboard
st.title("Network Traffic Monitor")
st.write("Real-time network packet analysis")

# Start/Stop button
if not st.session_state.monitor_started:
    if st.button("Start Monitoring"):
        st.session_state.monitor_started = True
        threading.Thread(target=start_monitoring, daemon=True).start()
        st.experimental_rerun()
else:
    st.success("Monitoring active")

# Stats containers
col1, col2, col3 = st.columns(3)
with col1:
    st.metric("Total Packets", len(st.session_state.events))
with col2:
    protocols = set(e["protocol"] for e in st.session_state.events)
    st.metric("Unique Protocols", len(protocols))
with col3:
    total_size = sum(e["size"] for e in st.session_state.events)
    st.metric("Total Data", f"{total_size/1024:.2f} KB")

# Update events from queue
while not event_queue.empty():
    try:
        event = event_queue.get_nowait()
        st.session_state.events.insert(0, event)  # Add to start of list
    except queue.Empty:
        break

# Show recent events table
st.subheader("Recent Network Events")
if st.session_state.events:
    st.dataframe(
        st.session_state.events[:100],  # Show last 100 events
        column_config={
            "time": "Time",
            "protocol": "Protocol",
            "source": "Source",
            "destination": "Destination",
            "size": st.column_config.NumberColumn("Size (bytes)")
        },
        hide_index=True,
    )
else:
    st.info("Waiting for network events...")

# Auto-refresh
time.sleep(0.1)  # Small delay to prevent too frequent updates
st.experimental_rerun()

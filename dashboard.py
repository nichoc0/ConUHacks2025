import streamlit as st
import requests

API_BASE_URL = "http://127.0.0.1:8080/api"

st.title("Network Security Monitor Dashboard")
st.markdown("This dashboard displays network events retrieved from MongoDB. Each event can be expanded to show details along with a placeholder report.")

# Fetch normal events
st.header("Normal Network Events")
try:
    response_normal = requests.get(f"{API_BASE_URL}/events")
    response_normal.raise_for_status()
    normal_events = response_normal.json()
except Exception as e:
    st.error(f"Error fetching normal events: {e}")
    normal_events = []

if normal_events:
    for event in normal_events:
        # Assuming MongoDB adds an '_id' field to the document
        event_id = event.get("_id", "N/A")
        with st.expander(f"{event.get('protocol', 'N/A')} event from {event.get('source', 'N/A')} to {event.get('destination', 'N/A')} at {event.get('timestamp', 'N/A')}"):
            st.json(event)
            if st.button(f"Generate Report for {event_id}", key=f"normal_{event_id}"):
                payload = {"event_id": event_id}
                try:
                    report_response = requests.post(f"{API_BASE_URL}/report", json=payload)
                    report_response.raise_for_status()
                    report = report_response.json().get("report", "No report")
                    st.success(report)
                except Exception as ex:
                    st.error(f"Error generating report: {ex}")
else:
    st.write("No normal network events found.")

# Fetch suspicious events
st.header("Suspicious Network Events")
try:
    response_suspicious = requests.get(f"{API_BASE_URL}/suspicious")
    response_suspicious.raise_for_status()
    suspicious_events = response_suspicious.json()
except Exception as e:
    st.error(f"Error fetching suspicious events: {e}")
    suspicious_events = []

if suspicious_events:
    for event in suspicious_events:
        event_id = event.get("_id", "N/A")
        with st.expander(f"{event.get('protocol', 'N/A')} event from {event.get('source', 'N/A')} to {event.get('destination', 'N/A')} at {event.get('timestamp', 'N/A')}"):
            st.json(event)
            if st.button(f"Generate Report for {event_id}", key=f"suspicious_{event_id}"):
                payload = {"event_id": event_id}
                try:
                    report_response = requests.post(f"{API_BASE_URL}/report", json=payload)
                    report_response.raise_for_status()
                    report = report_response.json().get("report", "No report")
                    st.success(report)
                except Exception as ex:
                    st.error(f"Error generating report: {ex}")
else:
    st.write("No suspicious network events found.")

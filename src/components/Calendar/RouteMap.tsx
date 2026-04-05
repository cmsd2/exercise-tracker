import { useMemo } from "react";
import { MapContainer, TileLayer, Polyline, useMap } from "react-leaflet";
import { LatLngBoundsExpression } from "leaflet";
import "leaflet/dist/leaflet.css";

interface RouteMapProps {
  coordinates: [number, number][];
}

function FitBounds({ bounds }: { bounds: LatLngBoundsExpression }) {
  const map = useMap();
  useMemo(() => {
    map.fitBounds(bounds, { padding: [30, 30] });
  }, [map, bounds]);
  return null;
}

export function RouteMap({ coordinates }: RouteMapProps) {
  const bounds = useMemo(
    () => coordinates as LatLngBoundsExpression,
    [coordinates],
  );

  return (
    <MapContainer
      bounds={bounds}
      style={{
        height: 250,
        width: "100%",
        borderRadius: "var(--radius-sm)",
        marginTop: "var(--spacing-sm)",
      }}
      scrollWheelZoom={true}
      attributionControl={true}
    >
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      <Polyline
        positions={coordinates}
        pathOptions={{ color: "#0A84FF", weight: 3 }}
      />
      <FitBounds bounds={bounds} />
    </MapContainer>
  );
}

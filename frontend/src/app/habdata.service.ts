import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http'
import { Observable } from 'rxjs';

export interface HABUpdate {
    trip: number;
    time_logged: string;
    day_logged: number;
    month_logged: number;
    year_logged: number;
    hour: number;
    minute: number;
    fixquality: number;
    speed: number;
    angle: number;
    lon: number;
    lat: number;
    altitude: number;
    temp: number;
}

@Injectable({
  providedIn: 'root'
})
export class HabdataService {

  url: string = 'http://api.stachab.org/';

  constructor(private http: HttpClient) { }

  getUpdates(trip: number): Observable<HABUpdate[]> {
    let updates = this.http.get<HABUpdate[]>(this.url + String(trip));
    
    return updates;
  } 
}

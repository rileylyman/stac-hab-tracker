import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { HabdataService, HABUpdate } from '../habdata.service';

@Component({
  selector: 'app-launches',
  templateUrl: './launches.component.html',
  styleUrls: ['./launches.component.scss']
})
export class LaunchesComponent implements OnInit {

  trip$: number;
  updates$: HABUpdate[];

  constructor(private route: ActivatedRoute, private data: HabdataService) { 
    this.route.params.subscribe( 
      params => this.trip$ = params.trip
    );
  }

  ngOnInit() {
    this.data.getUpdates(this.trip$).subscribe(
      data => 
      {
        for (let i = 0; i < data.length; i++) {

          var d  = Math.floor (data[i].lon / 100.0);
          var m  = (data[i].lon-(d*100));
          
          data[i].lon = -(d + m/60.0);

          d  = Math.floor (data[i].lat / 100.0);
          m  = (data[i].lat-(d*100));

          data[i].lat = d + m/60.0;
        }
        this.updates$ = data;
      }
    );
  }

}
